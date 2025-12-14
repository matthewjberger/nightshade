use crate::ecs::animation::components::{
    AnimationProperty, AnimationValue, sample_animation_channel,
};
use crate::ecs::world::{ANIMATION_PLAYER, World};
use std::collections::HashMap;

#[tracing::instrument(skip(world), name = "animation")]
pub fn update_animation_players(world: &mut World, delta_time: f32) {
    let entities_with_animations: Vec<_> = world.query_entities(ANIMATION_PLAYER).collect();

    for entity in entities_with_animations {
        if let Some(animation_player) = world.get_animation_player_mut(entity) {
            animation_player.update(delta_time);
        }
    }
}

pub fn apply_animations(world: &mut World) {
    let entities_with_animations: Vec<_> = world.query_entities(ANIMATION_PLAYER).collect();

    for entity in entities_with_animations {
        let animation_data = if let Some(player) = world.get_animation_player(entity) {
            if !player.playing {
                continue;
            }

            if let Some(clip) = player.get_current_clip() {
                let time = player.time;
                let node_mapping = player.node_index_to_entity.clone();
                Some((clip.clone(), time, node_mapping))
            } else {
                None
            }
        } else {
            None
        };

        if let Some((clip, time, node_mapping)) = animation_data {
            let mut entity_animation_values: HashMap<_, AnimationAccumulator> = HashMap::new();

            for channel in &clip.channels {
                if let Some(&target_entity) = node_mapping.get(&channel.target_node)
                    && let Some(value) = sample_animation_channel(channel, time)
                {
                    let entry = entity_animation_values
                        .entry(target_entity)
                        .or_insert_with(AnimationAccumulator::default);

                    match (channel.target_property, value) {
                        (AnimationProperty::Translation, AnimationValue::Vec3(translation)) => {
                            entry.translation = Some(translation);
                        }
                        (AnimationProperty::Rotation, AnimationValue::Quat(rotation)) => {
                            entry.rotation = Some(rotation);
                        }
                        (AnimationProperty::Scale, AnimationValue::Vec3(scale)) => {
                            entry.scale = Some(scale);
                        }
                        _ => {}
                    }
                }
            }

            for (target_entity, values) in entity_animation_values {
                if let Some(transform) = world.mutate_local_transform(target_entity) {
                    if let Some(translation) = values.translation {
                        transform.translation = translation;
                    }
                    if let Some(rotation) = values.rotation {
                        transform.rotation = rotation.normalize();
                    }
                    if let Some(scale) = values.scale {
                        transform.scale = scale;
                    }
                }
            }
        }
    }
}

#[derive(Default)]
struct AnimationAccumulator {
    translation: Option<nalgebra_glm::Vec3>,
    rotation: Option<nalgebra_glm::Quat>,
    scale: Option<nalgebra_glm::Vec3>,
}
