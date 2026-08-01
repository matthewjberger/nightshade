use crate::components::{Dying, Pace, Phase, Playing, Rig, Stride};
use crate::data::districts::{
    CLIP_DIE, CLIP_DRIVE, CLIP_IDLE, CLIP_SPRINT, CLIP_STATIC, CLIP_WALK,
};
use crate::data::settings::Settings;
use crate::resources::Player;
use nightshade::prelude::*;
use nightshade_animation::contract::{AnimationPlayer, AnimationProperty, AnimationSamplerOutput};
use nightshade_config::contract::Config;
use nightshade_scene::prelude::LocalTransform;

type Rigged<'world, 'item> = nightshade::ecs::system_param::Query<
    'world,
    (
        &'item Rig,
        &'item Stride,
        &'item Phase,
        Option<&'item Pace>,
        Option<&'item Dying>,
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
    player: Res<Player>,
    config: Res<Config<Settings>>,
    rigged: Rigged<'_, '_>,
    mut phases: Query<(&mut Phase,)>,
    mut playing: Query<(&mut Playing,)>,
    mut players: Query<(&mut AnimationPlayer,)>,
    mut nodes: Query<(&mut LocalTransform,)>,
) {
    let stroll = config.number(Settings::Stroll);
    let mut choices = Vec::new();
    let mut moved = Vec::new();
    rigged.for_each(|entity, (rig, stride, phase, pace, dying)| {
        let Some(rig) = rig.0 else {
            return;
        };
        let walking = (stride.0 - phase.0).abs() > 1e-4;
        if walking {
            moved.push((entity, stride.0));
        }
        let hero = Some(entity) == player.body;
        let (clip, speed, looping) = if dying.is_some_and(|dying| dying.0 > 0.0) {
            (CLIP_DIE, 1.0, false)
        } else if hero && player.car.is_some() {
            (CLIP_DRIVE, 1.0, true)
        } else if walking {
            let hurrying = pace.is_some_and(|pace| pace.0 > stroll + 0.1);
            if (hero && player.sprinting) || hurrying {
                (CLIP_SPRINT, 1.2, true)
            } else {
                (CLIP_WALK, 1.3, true)
            }
        } else {
            (CLIP_IDLE, 1.0, true)
        };
        choices.push((rig, clip, speed, looping));
    });
    for (entity, stride) in moved {
        phases.for_entity(entity, |(phase,)| phase.0 = stride);
    }
    let mut rests = Vec::new();
    for (rig, clip, speed, looping) in choices {
        let mut current = None;
        playing.for_entity(rig, |(held,)| current = Some(held.0));
        if current == Some(clip) {
            continue;
        }
        playing.for_entity(rig, |(held,)| held.0 = clip);
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
