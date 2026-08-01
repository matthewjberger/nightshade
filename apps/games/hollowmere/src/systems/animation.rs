use crate::components::{
    Cooldowns, Dying, Flinch, Health, Job, Limb, Phase, Pivot, Reach, Rig, Rising, Sprint, Stride,
    Strike, Task,
};
use crate::data::{
    CLIP_ATTACK, CLIP_DIE, CLIP_IDLE, CLIP_SIT, CLIP_SPRINT, CLIP_STATIC, CLIP_WALK,
};
use crate::queries::is_alive;
use crate::theme;
use nightshade::prelude::*;
use nightshade_animation::contract::{AnimationPlayer, AnimationProperty, AnimationSamplerOutput};
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::{ChildOf, LocalTransform};
use std::collections::HashMap;

const PI: f32 = std::f32::consts::PI;

type Rigged<'world, 'item> = nightshade::ecs::system_param::Query<
    'world,
    (
        &'item Rig,
        &'item Stride,
        &'item Phase,
        &'item Health,
        &'item Strike,
        Option<&'item Task>,
        Option<&'item Sprint>,
        Option<&'item Cooldowns>,
    ),
>;

struct RestPose {
    node: Entity,
    translation: Option<Vec3>,
    rotation: Option<nalgebra_glm::Quat>,
    scale: Option<Vec3>,
}

fn rest_pose(player: &AnimationPlayer) -> Vec<RestPose> {
    let mut rests: Vec<RestPose> = Vec::new();
    let Some(clip) = player.clips.get(CLIP_STATIC) else {
        return rests;
    };
    for channel in &clip.channels {
        let Some(&node) = player.node_index_to_entity.get(&channel.target_node) else {
            continue;
        };
        let rest = match rests.iter_mut().find(|rest| rest.node == node) {
            Some(rest) => rest,
            None => {
                rests.push(RestPose {
                    node,
                    translation: None,
                    rotation: None,
                    scale: None,
                });
                rests.last_mut().expect("just pushed")
            }
        };
        match (&channel.target_property, &channel.sampler.output) {
            (AnimationProperty::Translation, AnimationSamplerOutput::Vec3(values)) => {
                rest.translation = values.first().copied();
            }
            (AnimationProperty::Scale, AnimationSamplerOutput::Vec3(values)) => {
                rest.scale = values.first().copied();
            }
            (AnimationProperty::Rotation, AnimationSamplerOutput::Quat(values)) => {
                rest.rotation = values.first().copied();
            }
            _ => {}
        }
    }
    rests
}

pub fn pose(
    rigged: Rigged<'_, '_>,
    mut phases: Query<(&mut Phase,)>,
    mut players: Query<(&mut AnimationPlayer,)>,
    mut nodes: Query<(&mut LocalTransform,)>,
) {
    let mut choices = Vec::new();
    let mut moved = Vec::new();
    rigged.for_each(
        |entity, (rig, stride, phase, health, strike, task, sprint, cooldowns)| {
            let Some(rig) = rig.0 else {
                return;
            };
            let walking = (stride.0 - phase.0).abs() > 1e-4;
            if walking {
                moved.push((entity, stride.0));
            }
            let (clip, speed, looping) = if !is_alive(health.0) {
                (CLIP_DIE, 1.0, false)
            } else if strike.0 > 0.0 {
                (CLIP_ATTACK, 1.3, false)
            } else if walking {
                let sprinting = sprint.is_some_and(|sprint| sprint.0 > 0.0);
                if sprinting {
                    (CLIP_SPRINT, 1.2, true)
                } else if cooldowns.is_some() {
                    (CLIP_WALK, 1.35, true)
                } else {
                    (CLIP_WALK, 1.15, true)
                }
            } else if task.is_some_and(|task| task.0 == Job::Resting) {
                (CLIP_SIT, 1.0, true)
            } else {
                (CLIP_IDLE, 1.0, true)
            };
            choices.push((rig, clip, speed, looping));
        },
    );
    for (entity, stride) in moved {
        phases.for_entity(entity, |(phase,)| phase.0 = stride);
    }
    let mut rests = Vec::new();
    for (rig, clip, speed, looping) in choices {
        players.for_entity(rig, |(player,)| {
            if player.current_clip == Some(clip) && player.clips.len() > clip {
                return;
            }
            if player.clips.len() <= clip {
                return;
            }
            rests.extend(rest_pose(player));
            player.current_clip = Some(clip);
            player.time = 0.0;
            player.speed = speed;
            player.looping = looping;
            player.playing = true;
        });
    }
    for rest in rests {
        nodes.for_entity(rest.node, |(transform,)| {
            if let Some(translation) = rest.translation {
                transform.translation = translation;
            }
            if let Some(rotation) = rest.rotation {
                transform.rotation = rotation;
            }
            if let Some(scale) = rest.scale {
                transform.scale = scale;
            }
        });
    }
}

pub fn hooves(
    bodies: Query<(&Stride, &Phase, &Health), Without<Rig>>,
    legs: Query<(&Limb, &Pivot, &Reach, &ChildOf)>,
    mut phases: Query<(&mut Phase,)>,
    mut transforms: Query<(&mut LocalTransform,)>,
) {
    let mut strides: HashMap<Entity, f32> = HashMap::new();
    bodies.for_each(|entity, (stride, phase, health)| {
        if is_alive(health.0) && (stride.0 - phase.0).abs() > 1e-4 {
            strides.insert(entity, stride.0);
        }
    });
    if strides.is_empty() {
        return;
    }
    let mut poses = Vec::new();
    legs.for_each(|entity, (limb, pivot, reach, parent)| {
        if let Some(stride) = strides.get(&parent.0) {
            let angle = (stride * theme::WALK_CYCLE).sin() * theme::WALK_SWING * limb.0;
            poses.push((entity, pivot.0, reach.0, angle));
        }
    });
    for (entity, pivot, reach, angle) in poses {
        transforms.for_entity(entity, |(transform,)| {
            let rotation = nalgebra_glm::quat_angle_axis(angle, &Vec3::x());
            transform.rotation = rotation;
            transform.translation = pivot + nalgebra_glm::quat_rotate_vec3(&rotation, &reach);
        });
    }
    for (entity, stride) in strides {
        phases.for_entity(entity, |(phase,)| phase.0 = stride);
    }
}

pub fn strikes(time: Res<Time>, mut striking: Query<(&mut Strike,)>) {
    let delta_time = time.delta_time;
    striking.for_each_mut(|_entity, (strike,)| {
        if strike.0 > 0.0 {
            strike.0 = (strike.0 - delta_time).max(0.0);
        }
    });
}

pub fn flinches(
    time: Res<Time>,
    flinching: Query<(&Flinch,)>,
    mut timers: Query<(&mut Flinch,)>,
    mut transforms: Query<(&mut LocalTransform,)>,
) {
    let delta_time = time.delta_time;
    let mut active = Vec::new();
    flinching.for_each(|entity, (flinch,)| {
        if flinch.0 > 0.0 {
            active.push((entity, flinch.0));
        }
    });
    for (entity, remaining) in active {
        let next = (remaining - delta_time).max(0.0);
        timers.for_entity(entity, |(flinch,)| flinch.0 = next);
        let progress = 1.0 - next / theme::FLINCH_SECONDS;
        let squash = (progress * PI).sin() * 0.18;
        transforms.for_entity(entity, |(transform,)| {
            transform.scale = Vec3::new(1.0 + squash, 1.0 - squash * 1.3, 1.0 + squash);
        });
    }
}

pub fn falls(
    time: Res<Time>,
    dying: Query<(&Dying, Option<&Rig>)>,
    mut timers: Query<(&mut Dying,)>,
    mut transforms: Query<(&mut LocalTransform,)>,
) {
    let delta_time = time.delta_time;
    let mut active = Vec::new();
    dying.for_each(|entity, (dying, rig)| {
        if dying.0 > 0.0 {
            active.push((entity, dying.0, rig.is_some()));
        }
    });
    for (entity, remaining, rigged) in active {
        let next = (remaining - delta_time).max(0.0);
        timers.for_entity(entity, |(dying,)| dying.0 = next);
        let progress = 1.0 - next / theme::DYING_SECONDS;
        transforms.for_entity(entity, |(transform,)| {
            if next <= 0.0 {
                transform.translation.y -= theme::SINK_DEPTH;
                transform.scale = Vec3::new(1.0, 1.0, 1.0);
                return;
            }
            if !rigged {
                let yaw = yaw_of(transform.rotation);
                let tilt = nalgebra_glm::quat_angle_axis(-progress.min(1.0) * 1.4, &Vec3::x());
                transform.rotation = nalgebra_glm::quat_angle_axis(yaw, &Vec3::y()) * tilt;
            }
        });
    }
}

fn yaw_of(rotation: nalgebra_glm::Quat) -> f32 {
    let forward = nalgebra_glm::quat_rotate_vec3(&rotation, &Vec3::new(0.0, 0.0, -1.0));
    (-forward.x).atan2(-forward.z)
}

pub fn rises(
    time: Res<Time>,
    rising: Query<(&Rising,)>,
    mut timers: Query<(&mut Rising,)>,
    mut transforms: Query<(&mut LocalTransform,)>,
) {
    let delta_time = time.delta_time;
    let mut active = Vec::new();
    rising.for_each(|entity, (rising,)| {
        if rising.0 > 0.0 {
            active.push((entity, rising.0));
        }
    });
    for (entity, remaining) in active {
        let next = (remaining - delta_time).max(0.0);
        timers.for_entity(entity, |(rising,)| rising.0 = next);
        let grown = 1.0 - next / theme::RISING_SECONDS;
        let scale = 0.2 + 0.8 * grown.clamp(0.0, 1.0);
        transforms.for_entity(entity, |(transform,)| {
            transform.scale = Vec3::new(scale, scale, scale);
        });
    }
}
