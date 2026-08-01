use crate::components::{Angry, Beast, Goal, Lair, Roam};
use crate::data::beasts::BEASTS;
use crate::data::settings::Settings;
use crate::queries::facing_offset;
use crate::resources::Quarry;
use crate::theme;
use blocker_blocks::queries::{footing, step_holds};
use blocker_blocks::resources::VoxelWorld;
use blocker_figure::components::{Eye, Gait, Gaze, Heading, Pace};
use blocker_figure::queries::read_the_hero;
use blocker_figure::resources::Hero;
use blocker_player::data::ask::Ask;
use nalgebra_glm::{Vec2, Vec3};
use nightshade::prelude::*;
use nightshade_config::prelude::Config;
use nightshade_health::{Health, queries::fallen};
use nightshade_input::Actions;
use nightshade_platform::prelude::Time;
use nightshade_random::Random;
use nightshade_random::commands::between;
use nightshade_scene::prelude::LocalTransform;

type Herds<'world, 'item> = nightshade::ecs::system_param::Query<
    'world,
    (
        &'item Beast,
        &'item Lair,
        &'item mut Goal,
        &'item mut Roam,
        &'item mut Angry,
        &'item mut Heading,
        &'item mut LocalTransform,
    ),
>;

pub fn roam_the_beasts(
    time: Res<Time>,
    world: Res<VoxelWorld>,
    config: Res<Config<Settings>>,
    hero: Res<Hero>,
    mut random: ResMut<Random>,
    mut beasts: Herds<'_, '_>,
    mut strides: Query<(&mut Gait, &mut Pace)>,
    mut sighted: Query<(&Eye,)>,
) {
    let Some(eye) = read_the_hero(&hero, &mut sighted, |(place,)| place.0) else {
        return;
    };
    let delta = time.delta_time.min(0.05);
    let aggro = config.number(Settings::Aggro);
    let anger = config.number(Settings::Anger);
    let leash = config.number(Settings::Leash);
    let roam_radius = config.number(Settings::RoamRadius);
    let melee = config.number(Settings::Melee);
    let player_ground = Vec2::new(eye.x, eye.z);
    let mut stepped: Vec<(Entity, f32, bool)> = Vec::new();
    beasts.for_each_mut(
        |entity, (beast, lair, goal, roam, angry, heading, transform)| {
            let kind = &BEASTS[beast.0];
            let ground = Vec2::new(transform.translation.x, transform.translation.z);
            let player_distance = (player_ground - ground).magnitude();
            if kind.aggro > 0.0 && player_distance < aggro {
                angry.0 = anger;
            }
            angry.0 = (angry.0 - delta).max(0.0);
            let leashed = (ground - lair.0).magnitude() < leash;
            let hunting = angry.0 > 0.0 && leashed;
            let target = if hunting {
                player_ground
            } else {
                roam.0 -= delta;
                if roam.0 <= 0.0 {
                    let turn = between(&mut random, 0.0, std::f32::consts::TAU);
                    let reach = between(&mut random, 0.0, roam_radius);
                    goal.0 = lair.0 + Vec2::new(turn.cos(), turn.sin()) * reach;
                    roam.0 = between(
                        &mut random,
                        theme::BEAST_REST_MIN,
                        theme::BEAST_REST_MIN + theme::BEAST_REST,
                    );
                }
                goal.0
            };
            let to_target = target - ground;
            let distance = to_target.magnitude();
            let stop = if hunting { melee * 0.8 } else { 0.4 };
            let speed = kind.speed * if hunting { 1.0 } else { theme::BEAST_CALM_PACE };
            let mut moving = false;
            if distance > 1.0e-3 {
                heading.0 =
                    (to_target.x / distance).atan2(to_target.y / distance) + facing_offset(beast.0);
            }
            if distance > stop {
                let direction = to_target / distance;
                let step = (speed * delta).min(distance);
                let next = ground + direction * step;
                let stand = transform.translation.y.floor() as i32;
                if step_holds(&world, ground, next, stand) {
                    transform.translation.x = next.x;
                    transform.translation.z = next.y;
                    moving = true;
                }
            }
            stepped.push((entity, speed, moving));
            if let Some(ground_height) = footing(
                &world,
                transform.translation.x.floor() as i32,
                transform.translation.z.floor() as i32,
                transform.translation.y.floor() as i32,
            ) {
                let wanted_y = ground_height as f32 + 1.0;
                let share = (theme::BEAST_STEP * delta).clamp(0.0, 1.0);
                transform.translation.y += (wanted_y - transform.translation.y) * share;
            }
        },
    );
    for (entity, speed, moving) in stepped {
        strides.for_entity(entity, |(gait, pace)| {
            let wanted = if moving { 1.0 } else { 0.0 };
            pace.0 += (wanted - pace.0) * (theme::BEAST_PACE_EASE * delta).clamp(0.0, 1.0);
            gait.0 += speed * delta * theme::BEAST_GAIT_RATE * pace.0;
            if gait.0 > std::f32::consts::TAU {
                gait.0 -= std::f32::consts::TAU;
            }
        });
    }
}

pub fn target_nearest(
    actions: Res<Actions<Ask>>,
    config: Res<Config<Settings>>,
    hero: Res<Hero>,
    mut quarry: ResMut<Quarry>,
    beasts: Query<(&Beast, &Health, &LocalTransform)>,
    mut sighted: Query<(&Eye, &Gaze)>,
) {
    let Some((eye, gaze)) = read_the_hero(&hero, &mut sighted, |(place, ahead)| (place.0, ahead.0))
    else {
        return;
    };
    let forward = gaze;
    let target_range = config.number(Settings::TargetRange);
    let mut candidates: Vec<(u8, f32, Entity)> = Vec::new();
    beasts.for_each(|entity, (beast, health, transform)| {
        if fallen(health) {
            return;
        }
        let head = transform.translation + Vec3::new(0.0, BEASTS[beast.0].head, 0.0);
        let offset = head - eye;
        let distance = offset.magnitude();
        if distance >= target_range {
            return;
        }
        let aim = if distance > 1.0e-3 {
            forward.dot(&(offset / distance))
        } else {
            1.0
        };
        let tier = if aim > theme::TARGET_AIM_COSINE {
            0
        } else if aim > theme::TARGET_VIEW_COSINE {
            1
        } else {
            2
        };
        candidates.push((tier, distance, entity));
    });
    candidates.sort_by(|left, right| left.0.cmp(&right.0).then(left.1.total_cmp(&right.1)));
    if let Some(current) = quarry.0
        && !candidates.iter().any(|(_, _, entity)| *entity == current)
    {
        quarry.0 = None;
    }
    if actions.just_pressed(Ask::Target) {
        quarry.0 = match quarry.0 {
            Some(current) => {
                let index = candidates
                    .iter()
                    .position(|(_, _, entity)| *entity == current)
                    .unwrap_or(0);
                candidates
                    .get((index + 1) % candidates.len().max(1))
                    .map(|(_, _, entity)| *entity)
            }
            None => candidates.first().map(|(_, _, entity)| *entity),
        };
    }
}
