use crate::commands::hurt_player;
use crate::components::{
    Bled, Body, Floor, Health, HuskKind, HuskState, Interest, Moving, NextTile, Phase, Repath,
    Rest, Scream, Speed, Stride, Swing, Timer,
};
use crate::data::rules::*;
use crate::queries::{flat_distance, next_step, tile_center, tile_of};
use crate::resources::{AshfallResources, Screen, Senses};
use crate::tags::{Flashlight, Player};
use nightshade::prelude::*;
use nightshade_flashlight::queries::shining;
use nightshade_flashlight::{Battery, Lit};
use nightshade_physics::queries::physics_world_cast_ray;
use nightshade_physics::resources::PhysicsWorld;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::*;
use nightshade_senses::commands::make_a_noise;
use nightshade_senses::{Exposure, Noise, Noises};
use nightshade_ui_flash::Flash;

pub type Minds<'world, 'item> = Query<
    'world,
    (
        &'item HuskKind,
        &'item mut HuskState,
        &'item mut Health,
        &'item mut Timer,
        &'item mut Scream,
        &'item mut Bled,
        &'item mut Interest,
    ),
>;

pub type Bodies<'world, 'item> = Query<
    'world,
    (
        &'item mut Repath,
        &'item mut NextTile,
        &'item mut Stride,
        &'item Speed,
        &'item Floor,
        &'item mut Moving,
        &'item mut LocalTransform,
    ),
>;

nightshade::ecs::query_bundle! {
    pub struct Husks<'world, 'item> {
        pub minds: Minds<'world, 'item>,
        pub bodies: Bodies<'world, 'item>,
    }
}

pub type Wounded<'world, 'item> = Query<
    'world,
    (
        &'item mut HuskState,
        &'item mut Health,
        &'item mut Timer,
        &'item mut Moving,
        &'item LocalTransform,
    ),
>;

enum Order {
    Still,
    LieDown,
    StandUp(f32),
    Face(Vec3),
    Walk(Vec3),
}

pub fn update(
    time: Res<Time>,
    physics: Res<PhysicsWorld>,
    mut resources: ResMut<AshfallResources>,
    mut senses: ResMut<Senses>,
    mut noises: ResMut<Noises>,
    mut screen: ResMut<Flash>,
    mut commands: Commands,
    mut players: Query<(&LocalTransform, &Exposure), WithTag<Player>>,
    mut torches: Query<(&Battery, &Lit), WithTag<Flashlight>>,
    mut husks: Husks<'_, '_>,
) {
    let resources = &mut *resources;
    if resources.screen != Screen::Playing {
        return;
    }
    let delta = time.delta_time;
    let Some(body) = resources.player.body else {
        return;
    };
    let mut seen = None;
    players.for_entity(body, |(transform, exposure)| {
        seen = Some((transform.translation, exposure.0))
    });
    let Some((player_position, exposure)) = seen else {
        return;
    };

    let mut alight = false;
    torches.for_each_mut(|_entity, (battery, lit)| {
        alight = shining(lit) && battery.0 > 0.0;
    });
    let lit = if alight {
        HUSK_SIGHT_LIT
    } else {
        HUSK_SIGHT_DARK
    };
    let sight = lit * exposure;
    let blocked = &resources.run.blocked;
    let physics = &*physics;
    let heard_noises = noises.list.clone();

    let mut radio = 0.0_f32;
    let mut damage = 0.0_f32;
    let mut finished = Vec::new();
    let mut screams = Vec::new();
    let mut stains = Vec::new();

    let mut places: Vec<(Entity, Vec3)> = Vec::new();
    husks
        .bodies
        .for_each_mut(|entity, (_, _, _, _, _, _, transform)| {
            places.push((entity, transform.translation));
        });

    let mut orders: Vec<(Entity, bool, Order)> = Vec::new();
    husks.minds.for_each_mut(
        |entity, (kind, state, health, timer, scream, bled, interest)| {
            let profile = stats(*kind);
            let Some(here) = places
                .iter()
                .find(|(known, _)| *known == entity)
                .map(|(_, at)| *at)
            else {
                return;
            };
            let distance = flat_distance(&here, &player_position);
            if *state != HuskState::Dead {
                radio = radio.max((1.0 - distance / RADIO_RANGE).clamp(0.0, 1.0));
            }
            scream.0 = (scream.0 - delta).max(0.0);

            match *state {
                HuskState::Dead => {
                    timer.0 -= delta;
                    orders.push((entity, false, Order::LieDown));
                    if !bled.0 {
                        bled.0 = true;
                        stains.push(here);
                    }
                    if timer.0 <= 0.0 {
                        finished.push(entity);
                    }
                    return;
                }
                HuskState::Downed => {
                    timer.0 -= delta;
                    orders.push((entity, false, Order::LieDown));
                    if timer.0 <= 0.0 {
                        health.0 = profile.health * HUSK_DOWN_HEALTH;
                        *state = HuskState::Rising;
                        timer.0 = profile.rise;
                    }
                    return;
                }
                HuskState::Slumped => {
                    orders.push((entity, false, Order::LieDown));
                    let heard = heard_noises.iter().any(|noise| {
                        noise.loudness >= HUSK_ROUSE_NOISE
                            && flat_distance(&noise.at, &here) < noise.loudness
                    });
                    if distance < HUSK_ROUSE_RANGE || heard {
                        *state = HuskState::Rising;
                        timer.0 = profile.rise;
                    }
                    return;
                }
                HuskState::Rising => {
                    timer.0 -= delta;
                    let share = (timer.0 / profile.rise).clamp(0.0, 1.0);
                    orders.push((entity, false, Order::StandUp(share)));
                    if timer.0 <= 0.0 {
                        *state = HuskState::Chase;
                        timer.0 = 0.4;
                        if profile.screams && scream.0 <= 0.0 {
                            scream.0 = SCREAM_INTERVAL;
                            screams.push(here);
                        }
                    }
                    return;
                }
                HuskState::Stagger => {
                    timer.0 -= delta;
                    orders.push((entity, false, Order::Still));
                    if timer.0 <= 0.0 {
                        *state = HuskState::Chase;
                        timer.0 = 0.4;
                    }
                    return;
                }
                HuskState::Dormant | HuskState::Investigate => {
                    let seen = distance < sight * profile.sight
                        && line_is_clear(physics, entity, here, player_position);
                    if seen {
                        *state = HuskState::Chase;
                        timer.0 = 0.6;
                        interest.0 = None;
                        if profile.screams && scream.0 <= 0.0 {
                            scream.0 = SCREAM_INTERVAL;
                            screams.push(here);
                        }
                    } else {
                        let mut restart = false;
                        if let Some(heard) = loudest(&heard_noises, here) {
                            *state = HuskState::Investigate;
                            interest.0 = Some(heard);
                            timer.0 = INVESTIGATE_SECONDS;
                            restart = true;
                        }
                        let Some(target) = interest.0 else {
                            orders.push((entity, restart, Order::Still));
                            return;
                        };
                        timer.0 -= delta;
                        if timer.0 <= 0.0 || flat_distance(&here, &target) < INVESTIGATE_ARRIVAL {
                            *state = HuskState::Dormant;
                            interest.0 = None;
                            orders.push((entity, restart, Order::Still));
                            return;
                        }
                        orders.push((entity, restart, Order::Walk(target)));
                        return;
                    }
                }
                HuskState::Chase => {
                    if let Some(heard) = loudest(&heard_noises, here)
                        && distance > sight * profile.sight
                    {
                        interest.0 = Some(heard);
                    }
                }
            }

            if distance <= profile.reach {
                timer.0 -= delta;
                if timer.0 <= 0.0 {
                    damage += profile.damage;
                    timer.0 = profile.interval;
                }
                orders.push((entity, false, Order::Face(player_position)));
                return;
            }

            orders.push((entity, false, Order::Walk(player_position)));
        },
    );

    husks.bodies.for_each_mut(
        |entity, (repath, next_tile, stride, speed, floor, moving, transform)| {
            let Some((_, restart, order)) = orders.iter().find(|(known, ..)| *known == entity)
            else {
                return;
            };
            if *restart {
                repath.0 = 0.0;
            }
            match order {
                Order::Still => moving.0 = false,
                Order::LieDown => {
                    moving.0 = false;
                    transform.rotation =
                        nalgebra_glm::quat_angle_axis(-std::f32::consts::FRAC_PI_2, &Vec3::x());
                    transform.translation.y = floor.0 * 0.45;
                }
                Order::StandUp(share) => {
                    moving.0 = false;
                    let angle = -std::f32::consts::FRAC_PI_2 * share;
                    transform.rotation = nalgebra_glm::quat_angle_axis(angle, &Vec3::x());
                    transform.translation.y = floor.0 * (0.45 + 0.55 * (1.0 - share));
                }
                Order::Face(target) => {
                    moving.0 = false;
                    face(transform, *target);
                }
                Order::Walk(goal) => {
                    let goal = *goal;
                    let here = transform.translation;
                    repath.0 -= delta;
                    let direct = flat_distance(&here, &goal) < 6.0
                        && line_is_clear(physics, entity, here, goal);
                    let target = if direct {
                        next_tile.0 = None;
                        goal
                    } else {
                        let reached = next_tile.0.is_none_or(|next| {
                            flat_distance(&tile_center(next.0, next.1), &here) < 0.3
                        });
                        if repath.0 <= 0.0 || reached {
                            next_tile.0 = next_step(blocked, tile_of(&here), tile_of(&goal));
                            repath.0 = HUSK_REPATH_INTERVAL;
                        }
                        match next_tile.0 {
                            Some(next) => tile_center(next.0, next.1),
                            None => here,
                        }
                    };

                    let step = Vec3::new(target.x - here.x, 0.0, target.z - here.z);
                    let length = step.magnitude();
                    if length < 0.05 {
                        moving.0 = false;
                        return;
                    }
                    let direction = step / length;
                    let advance = (speed.0 * delta).min(length);
                    transform.translation += direction * advance;
                    transform.translation.y = floor.0;
                    transform.rotation =
                        nalgebra_glm::quat_angle_axis(direction.x.atan2(direction.z), &Vec3::y());
                    stride.0 += advance;
                    moving.0 = true;
                }
            }
        },
    );

    for entity in finished {
        commands.despawn_recursive(entity);
    }
    for at in screams {
        make_a_noise(&mut noises, at, SCREAM_LOUDNESS);
    }
    senses.stains.extend(stains);
    resources.run.radio = radio;
    if damage > 0.0 {
        hurt_player(&mut resources.player, &mut screen, damage);
    }
}

pub fn wound(
    state: &mut HuskState,
    health: &mut Health,
    timer: &mut Timer,
    moving: &mut Moving,
    damage: f32,
) -> bool {
    if *state == HuskState::Dead {
        return false;
    }
    if *state == HuskState::Downed {
        *state = HuskState::Dead;
        timer.0 = HUSK_ROT_SECONDS;
        moving.0 = false;
        return true;
    }
    health.0 -= damage;
    if health.0 <= 0.0 {
        *state = HuskState::Downed;
        timer.0 = HUSK_DOWN_SECONDS;
        moving.0 = false;
        false
    } else {
        *state = HuskState::Stagger;
        timer.0 = HUSK_STAGGER;
        moving.0 = false;
        false
    }
}

pub fn is_down(state: &HuskState) -> bool {
    matches!(state, HuskState::Downed | HuskState::Dead)
}

fn loudest(noises: &[Noise], here: Vec3) -> Option<Vec3> {
    let mut best: Option<(Vec3, f32)> = None;
    for noise in noises {
        let reach = flat_distance(&noise.at, &here);
        if reach > noise.loudness {
            continue;
        }
        let strength = noise.loudness - reach;
        if best.is_none_or(|(_, best)| strength > best) {
            best = Some((noise.at, strength));
        }
    }
    best.map(|(at, _)| at)
}

fn face(transform: &mut LocalTransform, target: Vec3) {
    let direction = Vec3::new(
        target.x - transform.translation.x,
        0.0,
        target.z - transform.translation.z,
    );
    if direction.magnitude() > 0.01 {
        transform.rotation =
            nalgebra_glm::quat_angle_axis(direction.x.atan2(direction.z), &Vec3::y());
    }
}

fn line_is_clear(physics: &PhysicsWorld, husk: Entity, from: Vec3, to: Vec3) -> bool {
    let start = from + Vec3::new(0.0, 0.5, 0.0);
    let end = to + Vec3::new(0.0, 0.4, 0.0);
    let path = end - start;
    let distance = path.magnitude();
    if distance < 0.01 {
        return true;
    }
    match physics_world_cast_ray(physics, start, path / distance, distance, Some(husk)) {
        None => true,
        Some(hit) => hit.distance >= distance - 0.5,
    }
}

pub fn animate(
    time: Res<Time>,
    mut husks: Query<(&HuskState, &Stride, &Moving)>,
    limbs: Query<(&Body, &Rest, &Swing, &Phase, &mut LocalTransform)>,
) {
    let clock = time.uptime_seconds as f32;
    let mut gaits = Vec::new();
    husks.for_each_mut(|entity, (state, stride, moving)| {
        gaits.push((entity, stride.0, moving.0, *state));
    });
    limbs.for_each(|_entity, (body, rest, swing, phase, transform)| {
        let Some(&(_, stride, moving, state)) = gaits.iter().find(|(entity, ..)| *entity == body.0)
        else {
            return;
        };
        if matches!(
            state,
            HuskState::Dead | HuskState::Downed | HuskState::Slumped
        ) {
            return;
        }
        let angle = if moving {
            (stride * 2.8 + phase.0).sin() * swing.0
        } else {
            (clock * 1.3 + phase.0).sin() * 0.06
        };
        transform.rotation = nalgebra_glm::quat_angle_axis(angle, &Vec3::x());
        transform.translation = rest.0;
    });
}
