use crate::commands::figures::spawn_person;
use crate::commands::flashes::score_hit;
use crate::components::{
    Body, Came, Cell, Corner, Dying, Heading, Idle, Pace, Stride, Velocity, Yaw,
};
use crate::data::districts::{DAY_LIFE, NIGHT_LIFE, PEOPLE, PEOPLE_PER_CELL};
use crate::data::look::{FLASH_BASE, NIGHT_FROM, NIGHT_UNTIL};
use crate::data::motion::{HURRY_CHANCE, PERSON_RADIUS, SPAWN_REACH};
use crate::data::settings::Settings;
use crate::queries::collide::{circle_push, hull_at, hull_holds_point};
use crate::queries::grid::{
    cell_of, corner_neighbors, corner_position, in_ring, surface_height, yaw_of,
};
use crate::queries::view::is_night;
use crate::resources::{City, Flashes, Hull, Player, Streaming};
use nightshade::ecs::Storage;
use nightshade::prelude::*;
use nightshade_config::contract::Config;
use nightshade_kit::prelude::model;
use nightshade_platform::prelude::Time;
use nightshade_random::prelude::*;
use nightshade_render::data::config::RendererState;
use nightshade_scene::prelude::LocalTransform;
use std::collections::HashMap;

const ARRIVE: f32 = 0.35;

type Walkers<'world, 'item> = nightshade::ecs::system_param::Query<
    'world,
    (
        &'item mut Corner,
        &'item mut Came,
        &'item Pace,
        &'item mut Idle,
        &'item mut Dying,
        &'item mut Stride,
        &'item mut LocalTransform,
        &'item mut Cell,
    ),
>;

pub fn populate(
    city: Res<City>,
    player: Res<Player>,
    streaming: Res<Streaming>,
    renderer_state: Res<RendererState>,
    config: Res<Config<Settings>>,
    mut random: ResMut<Random>,
    storage: &mut Storage,
) {
    let Some(center) = streaming.center else {
        return;
    };
    let crowd = config.number(Settings::Crowd);
    let stroll = config.number(Settings::Stroll);
    let hurry = config.number(Settings::Hurry);
    let feet = player
        .body
        .and_then(|body| storage.get::<LocalTransform>(body))
        .map(|transform| transform.translation)
        .unwrap_or_default();
    let mut counts: HashMap<[i32; 2], usize> = HashMap::new();
    for (_, (cell, _)) in storage.query_ref::<(&Cell, &Corner)>().iter() {
        *counts.entry(cell.0).or_default() += 1;
    }
    let night = is_night(renderer_state.day_night.hour, NIGHT_FROM, NIGHT_UNTIL);
    for (key, layout) in &city.cells {
        if !in_ring(*key, center, streaming.life) {
            continue;
        }
        let district = layout.district as usize;
        let life = if night {
            NIGHT_LIFE[district]
        } else {
            DAY_LIFE[district]
        };
        let wanted = (PEOPLE_PER_CELL[district] as f32 * life * crowd).round() as usize;
        if counts.get(key).copied().unwrap_or(0) >= wanted {
            continue;
        }
        let node = [key[0], key[1], whole(&mut random, 0, 3)];
        let position = corner_position(node);
        if (position - feet).norm() < SPAWN_REACH {
            continue;
        }
        let neighbors = corner_neighbors(node);
        let next = neighbors[index(&mut random, neighbors.len())];
        let Some(name) = pick(&mut random, &PEOPLE) else {
            continue;
        };
        let pace = if chance(&mut random, HURRY_CHANCE) {
            stroll * hurry
        } else {
            stroll
        };
        let progress = between(&mut random, 0.1, 0.9);
        spawn_person(storage, model(name), node, next, progress, pace, *key);
    }
}

pub fn wander(
    time: Res<Time>,
    player: Res<Player>,
    streaming: Res<Streaming>,
    config: Res<Config<Settings>>,
    mut random: ResMut<Random>,
    mut flashes: ResMut<Flashes>,
    mut commands: Commands,
    cars: Query<(&Heading, &Body, &Yaw, &LocalTransform, &Velocity)>,
    mut hero: Query<(&LocalTransform,)>,
    mut walkers: Walkers<'_, '_>,
) {
    let Some(center) = streaming.center else {
        return;
    };
    let keep = streaming.keep;
    let life = streaming.life;
    let delta_time = time.delta_time;
    let idle_chance = config.number(Settings::IdleChance);
    let idle_seconds = config.number(Settings::IdleSeconds);
    let hit_speed = config.number(Settings::HitSpeed);
    let dying_seconds = config.number(Settings::DyingSeconds);
    let combo = config.number(Settings::ComboSeconds);
    let mut fast: Vec<Hull> = Vec::new();
    cars.for_each(|_, (_, body, yaw, transform, velocity)| {
        if velocity.0.norm() > hit_speed {
            fast.push(hull_at(&body.0, transform.translation, yaw.0));
        }
    });
    let mut feet_position = None;
    if player.car.is_none()
        && let Some(body) = player.body
    {
        hero.for_entity(body, |(transform,)| {
            feet_position = Some(transform.translation);
        });
    }
    let mut finished = Vec::new();
    let mut arrivals = Vec::new();

    walkers.for_each_mut(
        |entity, (corner, came, pace, idle, dying, stride, transform, cell)| {
            if !in_ring(cell.0, center, keep) {
                finished.push(entity);
                return;
            }
            if dying.0 > 0.0 {
                dying.0 -= delta_time;
                if dying.0 <= 0.0 {
                    finished.push(entity);
                }
                return;
            }
            let position = transform.translation;
            if fast
                .iter()
                .any(|hull| hull_holds_point(hull, position, PERSON_RADIUS))
            {
                dying.0 = dying_seconds;
                score_hit(&mut flashes, position, FLASH_BASE, combo);
                return;
            }
            if idle.0 > 0.0 {
                idle.0 -= delta_time;
                return;
            }
            if !in_ring(cell.0, center, life) && !in_ring([corner.0[0], corner.0[1]], center, life)
            {
                std::mem::swap(&mut corner.0, &mut came.0);
            }
            let goal = corner_position(corner.0);
            let delta = Vec3::new(goal.x - position.x, 0.0, goal.z - position.z);
            let distance = delta.norm();
            if distance < ARRIVE {
                arrivals.push((entity, corner.0, came.0));
                return;
            }
            let step = (pace.0 * delta_time).min(distance);
            let travel = delta / distance;
            let mut moved = position + travel * step;
            if let Some(feet_position) = feet_position {
                moved += circle_push(moved, PERSON_RADIUS, &[(feet_position, PERSON_RADIUS)]);
            }
            moved.y = surface_height(moved);
            transform.translation = moved;
            transform.rotation = nalgebra_glm::quat_angle_axis(yaw_of(delta), &Vec3::y());
            stride.0 += step;
            cell.0 = cell_of(moved);
        },
    );

    for (entity, node, before) in arrivals {
        let neighbors = corner_neighbors(node);
        let options: Vec<[i32; 3]> = neighbors
            .iter()
            .copied()
            .filter(|candidate| *candidate != before)
            .collect();
        let next = options[index(&mut random, options.len())];
        let rest = if chance(&mut random, idle_chance) {
            between(&mut random, idle_seconds * 0.3, idle_seconds)
        } else {
            0.0
        };
        walkers.for_entity(entity, |(corner, came, _, idle, _, _, _, _)| {
            came.0 = node;
            corner.0 = next;
            idle.0 = rest;
        });
    }
    for entity in finished {
        commands.queue(move |storage, _| {
            storage.despawn_recursive(entity);
        });
    }
}
