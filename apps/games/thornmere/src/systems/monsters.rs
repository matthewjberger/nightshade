use crate::commands::audio::cue;
use crate::commands::flash;
use crate::commands::{place_on_ground, walk_toward};
use crate::components::{
    Aggro, Chill, Cooldowns, Goal, Heading, Health, Home, Job, Leash, Speed, Stride, Target, Task,
    Wander, Worth,
};
use crate::data::{
    CHILL_MULTIPLIER, COMBAT_LINGER, PLAYER_SPACE, SEPARATION, SOUND_AGGRO, SOUND_PLAIN, STANDOFF,
};
use crate::queries::{flat_distance, footing, ground, heading_toward, is_alive};
use crate::resources::{Cues, Feel, Flashes, ThornmereResources};
use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_random::prelude::*;
use nightshade_scene::prelude::LocalTransform;
use std::collections::HashMap;

const WANDER_RADIUS: f32 = 6.0;
const AGGRO_COLOR: Vec4 = Vec4::new(1.0, 0.25, 0.2, 1.0);

type Movers<'world, 'item> = nightshade::ecs::system_param::Query<
    'world,
    (
        &'item mut Goal,
        &'item mut Wander,
        &'item mut Target,
        &'item mut Heading,
        &'item mut Stride,
        &'item mut LocalTransform,
    ),
    With<Worth>,
>;

type Facts<'world, 'item> = nightshade::ecs::system_param::Query<
    'world,
    (
        &'item Speed,
        &'item Home,
        &'item Aggro,
        &'item Leash,
        &'item Health,
        &'item Chill,
    ),
    With<Worth>,
>;

struct Fact {
    speed: f32,
    home: Vec3,
    aggro: f32,
    leash: f32,
    alive: bool,
    chilled: bool,
}

pub fn roam(
    time: Res<Time>,
    mut resources: ResMut<ThornmereResources>,
    mut cues: ResMut<Cues>,
    mut flashes: ResMut<Flashes>,
    mut feel: ResMut<Feel>,
    facts: Facts<'_, '_>,
    people: Query<(&Health, &LocalTransform, &Task)>,
    players: Query<(&Health, &LocalTransform), With<Cooldowns>>,
    victims: Query<(&Health, &LocalTransform), With<Target>>,
    mut movers: Movers<'_, '_>,
) {
    let delta_time = time.delta_time;
    let resources = &mut *resources;
    let player = resources.player;
    let random = &mut resources.random;

    let mut known: HashMap<Entity, Fact> = HashMap::new();
    facts.for_each(|entity, (speed, home, aggro, leash, health, chill)| {
        known.insert(
            entity,
            Fact {
                speed: speed.0,
                home: home.0,
                aggro: aggro.0,
                leash: leash.0,
                alive: is_alive(health.0),
                chilled: chill.0 > 0.0,
            },
        );
    });

    let mut prey: Vec<(Entity, Vec3)> = Vec::new();
    people.for_each(|entity, (health, transform, task)| {
        if is_alive(health.0) && task.0 != Job::Offline {
            prey.push((entity, transform.translation));
        }
    });
    players.for_each(|entity, (health, transform)| {
        if is_alive(health.0) {
            prey.push((entity, transform.translation));
        }
    });

    let mut spots: HashMap<Entity, (f32, Vec3)> = HashMap::new();
    victims.for_each(|entity, (health, transform)| {
        spots.insert(entity, (health.0, transform.translation));
    });

    let mut alarms = Vec::new();
    movers.for_each_mut(
        |entity, (goal, wander, target, heading, stride, transform)| {
            let Some(fact) = known.get(&entity) else {
                return;
            };
            if !fact.alive {
                return;
            }
            let pace = if fact.chilled {
                fact.speed * CHILL_MULTIPLIER
            } else {
                fact.speed
            };
            let position = transform.translation;

            if target.0.is_none() && fact.aggro > 0.0 {
                let mut nearest = None;
                let mut nearest_distance = fact.aggro;
                for (candidate, spot) in &prey {
                    let distance = flat_distance(position, *spot);
                    if distance < nearest_distance {
                        nearest_distance = distance;
                        nearest = Some(*candidate);
                    }
                }
                if let Some(candidate) = nearest {
                    target.0 = Some(candidate);
                    if Some(candidate) == player {
                        alarms.push(position);
                    }
                }
            }

            if let Some(target_entity) = target.0 {
                match spots.get(&target_entity) {
                    Some(&(target_health, spot))
                        if is_alive(target_health)
                            && flat_distance(fact.home, spot) < fact.leash =>
                    {
                        if flat_distance(position, spot) > STANDOFF {
                            walk_toward(transform, heading, stride, spot, pace, delta_time);
                        } else {
                            heading.0 = heading_toward(position, spot);
                            place_on_ground(transform, heading, stride);
                        }
                        return;
                    }
                    _ => {
                        target.0 = None;
                        goal.0 = fact.home;
                    }
                }
            }

            wander.0 -= delta_time;
            if wander.0 <= 0.0 {
                wander.0 = between(random, 2.0, 6.0);
                let angle = random.f32() * std::f32::consts::TAU;
                let radius = random.f32() * WANDER_RADIUS;
                goal.0 = ground(Vec2::new(
                    fact.home.x + angle.cos() * radius,
                    fact.home.z + angle.sin() * radius,
                ));
            }
            walk_toward(transform, heading, stride, goal.0, pace * 0.45, delta_time);
        },
    );

    for position in alarms {
        flash(&mut flashes, position, "!".to_string(), AGGRO_COLOR, 2.0);
        cue(&mut cues, SOUND_AGGRO, SOUND_PLAIN);
        feel.combat = feel.combat.max(COMBAT_LINGER);
    }
}

pub fn space(
    players: Query<(&LocalTransform,), With<Cooldowns>>,
    mut bodies: Query<(&Health, &mut LocalTransform), With<Worth>>,
) {
    let mut anchors = Vec::new();
    players.for_each(|_entity, (transform,)| anchors.push(transform.translation));
    let mut spots = Vec::new();
    bodies.for_each_mut(|entity, (health, transform)| {
        if is_alive(health.0) {
            spots.push((entity, transform.translation));
        }
    });
    let mut pushes: HashMap<Entity, Vec2> = HashMap::new();
    for (index, (entity, position)) in spots.iter().enumerate() {
        for (other, other_position) in spots.iter().skip(index + 1) {
            let delta = Vec2::new(position.x - other_position.x, position.z - other_position.z);
            let distance = delta.magnitude();
            if distance >= SEPARATION {
                continue;
            }
            let away = if distance > 1e-3 {
                delta / distance
            } else {
                Vec2::new(1.0, 0.0)
            };
            let push = away * (SEPARATION - distance) * 0.5;
            *pushes.entry(*entity).or_default() += push;
            *pushes.entry(*other).or_default() -= push;
        }
        for anchor in &anchors {
            let delta = Vec2::new(position.x - anchor.x, position.z - anchor.z);
            let distance = delta.magnitude();
            if distance >= PLAYER_SPACE {
                continue;
            }
            let away = if distance > 1e-3 {
                delta / distance
            } else {
                Vec2::new(0.0, 1.0)
            };
            *pushes.entry(*entity).or_default() += away * (PLAYER_SPACE - distance);
        }
    }
    for (entity, push) in pushes {
        bodies.for_entity(entity, |(_, transform)| {
            transform.translation.x += push.x * 0.5;
            transform.translation.z += push.y * 0.5;
            transform.translation.y = footing(transform.translation.x, transform.translation.z);
        });
    }
}
