use crate::ecs::animation::components::{
    AnimationClip, AnimationProperty, AnimationValue, sample_animation_channel,
};
use crate::ecs::world::{ANIMATION_PLAYER, MORPH_WEIGHTS, World};
use freecs::Entity;
use nalgebra_glm::{Quat, Vec3};
use std::collections::HashMap;

pub fn update_animation_players(world: &mut World, delta_time: f32) {
    let entities_with_animations: Vec<_> = world.query_entities(ANIMATION_PLAYER).collect();
    let _span =
        tracing::info_span!("animation", players = entities_with_animations.len()).entered();

    for entity in entities_with_animations {
        if let Some(animation_player) = world.get_animation_player_mut(entity) {
            animation_player.update(delta_time);
        }
    }
}

struct AnimationSampleData {
    to_clip: AnimationClip,
    to_time: f32,
    from_clip: Option<AnimationClip>,
    from_time: f32,
    blend_factor: f32,
    node_mapping: HashMap<usize, Entity>,
    bone_name_mapping: HashMap<String, Entity>,
}

pub fn apply_animations(world: &mut World) {
    let entities_with_animations: Vec<_> = world.query_entities(ANIMATION_PLAYER).collect();

    for entity in entities_with_animations {
        let animation_data = if let Some(player) = world.get_animation_player(entity) {
            if !player.playing {
                continue;
            }

            if let Some(to_clip) = player.get_current_clip() {
                let from_clip = player
                    .blend_from_clip
                    .and_then(|index| player.clips.get(index))
                    .cloned();

                Some(AnimationSampleData {
                    to_clip: to_clip.clone(),
                    to_time: player.time,
                    from_clip,
                    from_time: player.blend_from_time,
                    blend_factor: player.blend_factor,
                    node_mapping: player.node_index_to_entity.clone(),
                    bone_name_mapping: player.bone_name_to_entity.clone(),
                })
            } else {
                None
            }
        } else {
            None
        };

        if let Some(data) = animation_data {
            let to_values = sample_clip(
                &data.to_clip,
                data.to_time,
                &data.node_mapping,
                &data.bone_name_mapping,
            );

            let final_values = if let Some(from_clip) = &data.from_clip {
                let from_values = sample_clip(
                    from_clip,
                    data.from_time,
                    &data.node_mapping,
                    &data.bone_name_mapping,
                );
                blend_animation_values(&from_values, &to_values, data.blend_factor)
            } else {
                to_values
            };

            for (target_entity, values) in final_values {
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
                if let Some(weights) = values.morph_weights {
                    apply_morph_weights_to_entity_and_children(world, target_entity, &weights);
                }
            }
        }
    }
}

fn sample_clip(
    clip: &AnimationClip,
    time: f32,
    node_mapping: &HashMap<usize, Entity>,
    bone_name_mapping: &HashMap<String, Entity>,
) -> HashMap<Entity, AnimationAccumulator> {
    static LOGGED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    let should_log = !LOGGED.swap(true, std::sync::atomic::Ordering::Relaxed);

    let mut entity_values: HashMap<Entity, AnimationAccumulator> = HashMap::new();
    let mut found_count = 0usize;
    let mut not_found_count = 0usize;

    for channel in &clip.channels {
        let target_entity = if let Some(ref bone_name) = channel.target_bone_name {
            bone_name_mapping.get(bone_name).copied()
        } else {
            None
        }
        .or_else(|| node_mapping.get(&channel.target_node).copied());

        if target_entity.is_some() {
            found_count += 1;
        } else {
            not_found_count += 1;
            if should_log && not_found_count <= 3 {
                tracing::warn!(
                    "Animation channel target not found: node={}, bone_name={:?}",
                    channel.target_node,
                    channel.target_bone_name
                );
            }
        }

        if let Some(target_entity) = target_entity
            && let Some(value) = sample_animation_channel(channel, time)
        {
            let entry = entity_values.entry(target_entity).or_default();

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
                (AnimationProperty::MorphWeights, AnimationValue::Weights(weights)) => {
                    entry.morph_weights = Some(weights);
                }
                _ => {}
            }
        }
    }

    if should_log {
        tracing::info!(
            "Animation '{}': {} channels found targets, {} not found",
            clip.name,
            found_count,
            not_found_count
        );
    }

    entity_values
}

fn blend_animation_values(
    from: &HashMap<Entity, AnimationAccumulator>,
    to: &HashMap<Entity, AnimationAccumulator>,
    factor: f32,
) -> HashMap<Entity, AnimationAccumulator> {
    let mut result: HashMap<Entity, AnimationAccumulator> = HashMap::new();

    for (&entity, to_acc) in to {
        let from_acc = from.get(&entity);
        let blended = blend_accumulators(from_acc, to_acc, factor);
        result.insert(entity, blended);
    }

    for (&entity, from_acc) in from {
        if !to.contains_key(&entity) {
            result.insert(entity, from_acc.clone());
        }
    }

    result
}

fn blend_accumulators(
    from: Option<&AnimationAccumulator>,
    to: &AnimationAccumulator,
    factor: f32,
) -> AnimationAccumulator {
    let from = from.cloned().unwrap_or_default();

    AnimationAccumulator {
        translation: match (from.translation, to.translation) {
            (Some(a), Some(b)) => Some(a + (b - a) * factor),
            (None, Some(b)) => Some(b),
            (Some(a), None) => Some(a),
            (None, None) => None,
        },
        rotation: match (from.rotation, to.rotation) {
            (Some(a), Some(b)) => {
                let a_norm = a.normalize();
                let b_norm = b.normalize();
                let dot = a_norm.dot(&b_norm);
                let b_adj = if dot < 0.0 { -b_norm } else { b_norm };
                Some(nalgebra_glm::quat_slerp(&a_norm, &b_adj, factor).normalize())
            }
            (None, Some(b)) => Some(b),
            (Some(a), None) => Some(a),
            (None, None) => None,
        },
        scale: match (from.scale, to.scale) {
            (Some(a), Some(b)) => Some(a + (b - a) * factor),
            (None, Some(b)) => Some(b),
            (Some(a), None) => Some(a),
            (None, None) => None,
        },
        morph_weights: match (&from.morph_weights, &to.morph_weights) {
            (Some(a), Some(b)) => {
                let blended: Vec<f32> = a
                    .iter()
                    .zip(b.iter())
                    .map(|(a_val, b_val)| a_val + (b_val - a_val) * factor)
                    .collect();
                Some(blended)
            }
            (None, Some(b)) => Some(b.clone()),
            (Some(a), None) => Some(a.clone()),
            (None, None) => None,
        },
    }
}

#[derive(Default, Clone)]
struct AnimationAccumulator {
    translation: Option<Vec3>,
    rotation: Option<Quat>,
    scale: Option<Vec3>,
    morph_weights: Option<Vec<f32>>,
}

fn apply_morph_weights_to_entity_and_children(world: &mut World, entity: Entity, weights: &[f32]) {
    if world.entity_has_components(entity, MORPH_WEIGHTS)
        && let Some(morph_weights) = world.get_morph_weights_mut(entity)
    {
        morph_weights.weights = weights.to_vec();
    }

    let child_entities: Vec<Entity> = world
        .resources
        .children_cache
        .get(&entity)
        .cloned()
        .unwrap_or_default();

    for child in child_entities {
        apply_morph_weights_to_entity_and_children(world, child, weights);
    }
}
