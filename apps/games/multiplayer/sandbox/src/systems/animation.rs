use crate::components::{Avatar, Crouch, Grounded, Pace, Velocity};
use crate::data::{CLIP_CROUCH, CLIP_IDLE, CLIP_SPRINT, CLIP_STATIC, CLIP_WALK};
use crate::theme;
use nalgebra_glm::{Quat, Vec3};
use nightshade::prelude::*;
use nightshade_animation::contract::{AnimationPlayer, AnimationProperty, AnimationSamplerOutput};
use nightshade_scene::prelude::LocalTransform;

struct Rest {
    node: Entity,
    translation: Option<Vec3>,
    rotation: Option<Quat>,
    scale: Option<Vec3>,
}

fn rest_pose(player: &AnimationPlayer) -> Vec<Rest> {
    let mut rests: Vec<Rest> = Vec::new();
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
                rests.push(Rest {
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
    bodies: Query<(&Avatar, &Velocity, &Crouch, &Grounded)>,
    mut players: Query<(&mut AnimationPlayer,)>,
    mut nodes: Query<(&mut LocalTransform,)>,
) {
    let mut choices = Vec::new();
    bodies.for_each(|_entity, (avatar, velocity, crouch, grounded)| {
        let Some(rig) = avatar.0 else {
            return;
        };
        let speed = Vec3::new(velocity.0.x, 0.0, velocity.0.z).norm();
        let (clip, rate) = if crouch.0 > 0.5 {
            (CLIP_CROUCH, 1.0)
        } else if !grounded.0 {
            (CLIP_IDLE, 1.0)
        } else if speed > theme::RUN_SPEED {
            (CLIP_SPRINT, 1.0)
        } else if speed > theme::WALK_SPEED * 0.25 {
            (CLIP_WALK, (speed / theme::WALK_SPEED).clamp(0.6, 1.8))
        } else {
            (CLIP_IDLE, 1.0)
        };
        choices.push((rig, clip, rate));
    });

    let mut rests = Vec::new();
    for (rig, clip, rate) in choices {
        players.for_entity(rig, |(player,)| {
            if player.clips.len() <= clip {
                return;
            }
            if player.current_clip == Some(clip) {
                player.speed = rate;
                return;
            }
            rests.extend(rest_pose(player));
            player.current_clip = Some(clip);
            player.time = 0.0;
            player.speed = rate;
            player.looping = true;
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

pub fn amble(animals: Query<(&Avatar, &Pace)>, mut players: Query<(&mut AnimationPlayer,)>) {
    let mut rigs = Vec::new();
    animals.for_each(|_entity, (avatar, _pace)| {
        if let Some(rig) = avatar.0 {
            rigs.push(rig);
        }
    });
    for rig in rigs {
        players.for_entity(rig, |(player,)| {
            if player.current_clip == Some(0) || player.clips.is_empty() {
                return;
            }
            player.current_clip = Some(0);
            player.time = 0.0;
            player.speed = 1.0;
            player.looping = true;
            player.playing = true;
        });
    }
}
