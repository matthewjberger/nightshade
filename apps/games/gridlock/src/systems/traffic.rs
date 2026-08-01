use crate::commands::figures::spawn_car;
use crate::components::{Bend, Cell, Cruise, Heading, Next, Turn, Velocity, Yaw};
use crate::data::districts::{CARS, DAY_LIFE, District, NIGHT_LIFE, TRAFFIC_PER_CELL, TRUCKS};
use crate::data::look::{NIGHT_FROM, NIGHT_UNTIL};
use crate::data::motion::{
    GAP_WIDTH, SPAWN_CLEAR, STOP_GAP, TRAFFIC_ACCELERATION, TRAFFIC_BRAKING,
};
use crate::data::scale::{CELL, LANE, ROAD_LIFT, TILE};
use crate::data::settings::Settings;
use crate::queries::grid::{
    cell_of, crossing_open, heading_vector, in_ring, intersection_after, lane_position, right_of,
    tile_center, yaw_of,
};
use crate::queries::view::is_night;
use crate::resources::{City, Player, Streaming};
use nightshade::ecs::Storage;
use nightshade::prelude::*;
use nightshade_config::contract::Config;
use nightshade_kit::prelude::model;
use nightshade_platform::prelude::Time;
use nightshade_random::prelude::*;
use nightshade_render::data::config::RendererState;
use nightshade_scene::prelude::LocalTransform;
use std::collections::HashMap;

type Cars<'world, 'item> = nightshade::ecs::system_param::Query<
    'world,
    (
        &'item mut Heading,
        &'item mut Turn,
        &'item mut Next,
        &'item mut Bend,
        &'item Cruise,
        &'item mut Velocity,
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
    let density = config.number(Settings::Traffic);
    let cruise = config.number(Settings::Cruise);
    let spread = config.number(Settings::CruiseSpread);
    let feet = player
        .body
        .and_then(|body| storage.get::<LocalTransform>(body))
        .map(|transform| transform.translation)
        .unwrap_or_default();
    let mut counts: HashMap<[i32; 2], usize> = HashMap::new();
    for (_, (cell, _)) in storage.query_ref::<(&Cell, &Heading)>().iter() {
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
        let wanted = (TRAFFIC_PER_CELL[district] as f32 * life * density).round() as usize;
        if counts.get(key).copied().unwrap_or(0) >= wanted {
            continue;
        }
        let heading = whole(&mut random, 0, 3) as u8;
        let along = between(&mut random, TILE * 1.5, CELL - TILE * 1.5);
        let intersection = match heading {
            0 | 1 => *key,
            2 => [key[0] + 1, key[1]],
            _ => [key[0], key[1] + 1],
        };
        let position = lane_position(intersection, heading, along);
        if (position - feet).norm() < SPAWN_CLEAR {
            continue;
        }
        let pool: &[&str] = if layout.district == District::Industrial && chance(&mut random, 0.5) {
            &TRUCKS
        } else {
            &CARS
        };
        let Some(name) = pick(&mut random, pool) else {
            continue;
        };
        let pace = cruise * between(&mut random, 1.0 - spread, 1.0 + spread);
        spawn_car(storage, model(name), position, heading, pace, *key);
    }
}

pub fn flow(
    time: Res<Time>,
    player: Res<Player>,
    streaming: Res<Streaming>,
    config: Res<Config<Settings>>,
    mut random: ResMut<Random>,
    mut commands: Commands,
    all: Query<(&Heading, &LocalTransform, &Velocity)>,
    mut cars: Cars<'_, '_>,
    mut yaws: Query<(&mut Yaw,)>,
) {
    let Some(center) = streaming.center else {
        return;
    };
    let reach = streaming.life + 1;
    let mut gone = Vec::new();
    let delta_time = time.delta_time;
    let seconds = time.uptime_seconds as f32;
    let cycle = config.number(Settings::LightCycle);
    let turn_chance = config.number(Settings::TurnChance);
    let turn_speed = config.number(Settings::TurnSpeed);
    let gap_look = config.number(Settings::GapLook);
    let gap_stop = config.number(Settings::GapStop).min(gap_look - 1.0);
    let mut bodies: Vec<(Entity, Vec3, Vec3)> = Vec::new();
    all.for_each(|entity, (_, transform, velocity)| {
        bodies.push((entity, transform.translation, velocity.0));
    });
    let driven = player.car;
    let mut turned = Vec::new();

    cars.for_each_mut(
        |entity, (heading, turn, next, bend, cruise, velocity, transform, cell)| {
            if Some(entity) == driven {
                return;
            }
            if !in_ring(cell.0, center, reach) {
                gone.push(entity);
                return;
            }
            let direction = heading_vector(heading.0);
            let right = right_of(direction);
            let position = transform.translation;
            let speed = velocity.0.dot(&direction).max(0.0);
            let mut target = cruise.0;

            let center = tile_center(next.0, 0, 0);
            let to_center = (center - position).dot(&direction);
            if bend.0 < 0.0 {
                let stop_at = to_center - (TILE * 0.5 + STOP_GAP);
                let open = crossing_open(next.0, heading.0 % 2 == 1, seconds, cycle);
                if !open && stop_at > -0.6 {
                    target = target.min((stop_at * 2.0).max(0.0));
                }
                if turn.0 != heading.0 && to_center < TILE * 2.0 {
                    target = target.min(turn_speed);
                }
            } else {
                target = target.min(turn_speed);
            }

            for (other, other_position, other_velocity) in &bodies {
                if *other == entity {
                    continue;
                }
                let delta = other_position - position;
                let ahead = delta.dot(&direction);
                let lateral = delta.dot(&right).abs();
                if ahead > 0.5 && ahead < gap_look && lateral < GAP_WIDTH {
                    let other_speed = other_velocity.dot(&direction).max(0.0);
                    let room = ((ahead - gap_stop) / (gap_look - gap_stop)).clamp(0.0, 1.0);
                    target = target.min(room * cruise.0 + (1.0 - room) * other_speed.min(cruise.0));
                    if ahead < gap_stop {
                        target = 0.0;
                    }
                }
            }

            let speed = speed
                + (target - speed).clamp(
                    -TRAFFIC_BRAKING * delta_time,
                    TRAFFIC_ACCELERATION * delta_time,
                );
            let speed = speed.max(0.0);

            if bend.0 >= 0.0 {
                let after = heading_vector(turn.0);
                let entry = center - direction * (TILE * 0.5) + right * LANE;
                let exit = center + after * (TILE * 0.5) + right_of(after) * LANE;
                let control = center
                    + direction * (exit - center).dot(&direction)
                    + after * (entry - center).dot(&after);
                let length = (control - entry).norm() + (exit - control).norm();
                bend.0 += speed * delta_time / length.max(0.1);
                let progress = bend.0.min(1.0);
                let stay = 1.0 - progress;
                let point = entry * (stay * stay)
                    + control * (2.0 * stay * progress)
                    + exit * (progress * progress);
                let tangent =
                    (control - entry) * (2.0 * stay) + (exit - control) * (2.0 * progress);
                let tangent = if tangent.norm() > 1e-4 {
                    tangent.normalize()
                } else {
                    after
                };
                transform.translation = Vec3::new(point.x, ROAD_LIFT, point.z);
                transform.rotation = nalgebra_glm::quat_angle_axis(yaw_of(tangent), &Vec3::y());
                velocity.0 = tangent * speed;
                if bend.0 >= 1.0 {
                    transform.translation = Vec3::new(exit.x, ROAD_LIFT, exit.z);
                    heading.0 = turn.0;
                    bend.0 = -1.0;
                    next.0 = intersection_after(next.0, heading.0);
                    turned.push((entity, heading.0));
                }
            } else {
                let mut moved = position + direction * (speed * delta_time);
                let lane = center + right * LANE;
                if heading.0 % 2 == 0 {
                    moved.z = lane.z;
                } else {
                    moved.x = lane.x;
                }
                moved.y = ROAD_LIFT;
                transform.translation = moved;
                transform.rotation = nalgebra_glm::quat_angle_axis(yaw_of(direction), &Vec3::y());
                velocity.0 = direction * speed;
                let to_center = (center - moved).dot(&direction);
                if turn.0 != heading.0 && to_center <= TILE * 0.5 {
                    bend.0 = 0.0;
                } else if to_center < -TILE * 0.5 {
                    next.0 = intersection_after(next.0, heading.0);
                    turned.push((entity, heading.0));
                }
            }
            cell.0 = cell_of(transform.translation);
        },
    );

    for (entity, heading) in turned {
        let choice = if chance(&mut random, turn_chance) {
            if chance(&mut random, 0.5) {
                (heading + 1) % 4
            } else {
                (heading + 3) % 4
            }
        } else {
            heading
        };
        cars.for_entity(entity, |(_, turn, _, _, _, _, _, _)| turn.0 = choice);
        yaws.for_entity(entity, |(yaw,)| yaw.0 = yaw_of(heading_vector(heading)));
    }
    for entity in gone {
        commands.queue(move |storage, _| {
            storage.despawn_recursive(entity);
        });
    }
}
