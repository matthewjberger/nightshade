use crate::ecs::animation::components::{
    AnimationClip, AnimationProperty, AnimationValue, sample_animation_channel,
};
use crate::ecs::world::{ANIMATION_PLAYER, JOINT, MORPH_WEIGHTS, World};
use freecs::Entity;
use nalgebra_glm::{Quat, Vec3};
use std::collections::HashMap;

use crate::prelude::*;
pub fn update_animation_players(world: &mut World) {
    let _span = tracing::info_span!("animation").entered();
    let delta_time = world.resources.window.timing.delta_time;
    world
        .core
        .query_mut()
        .with(ANIMATION_PLAYER)
        .iter(|_, table, idx| {
            table.animation_player[idx].update(delta_time);
        });
}

pub fn apply_animations(world: &mut World) {
    // The GPU animation pass evaluates joint transforms for every playing,
    // non-`play_all` player, so the CPU only needs to apply joints for
    // `play_all` players. Morph weights and non-joint node targets are always
    // applied here.
    let mut per_player_values: Vec<(bool, HashMap<Entity, AnimationAccumulator>)> = Vec::new();
    world
        .core
        .query()
        .with(ANIMATION_PLAYER)
        .iter(|_, table, idx| {
            let player = &table.animation_player[idx];
            if !player.playing {
                return;
            }
            let joints_on_gpu = !player.play_all;
            let node_mapping = &player.node_index_to_entity;
            let bone_name_mapping = &player.bone_name_to_entity;

            let mut final_values: HashMap<Entity, AnimationAccumulator> = HashMap::new();
            if player.play_all {
                for clip in &player.clips {
                    merge_accumulators(
                        &mut final_values,
                        sample_clip(clip, player.time, node_mapping, bone_name_mapping),
                    );
                }
            } else if let Some(clip) = player.get_current_clip() {
                merge_accumulators(
                    &mut final_values,
                    sample_clip(clip, player.time, node_mapping, bone_name_mapping),
                );
            }

            if let Some(from_clip) = player
                .blend_from_clip
                .and_then(|index| player.clips.get(index))
            {
                let from_values = sample_clip(
                    from_clip,
                    player.blend_from_time,
                    node_mapping,
                    bone_name_mapping,
                );
                final_values =
                    blend_animation_values(&from_values, &final_values, player.blend_factor);
            }

            if !final_values.is_empty() {
                per_player_values.push((joints_on_gpu, final_values));
            }
        });

    for (joints_on_gpu, final_values) in per_player_values {
        for (target_entity, values) in final_values {
            let target_is_joint = world.core.entity_has_components(target_entity, JOINT);
            if !(joints_on_gpu && target_is_joint)
                && let Some(transform) = mutate_local_transform(world, target_entity)
            {
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

fn merge_accumulators(
    into: &mut HashMap<Entity, AnimationAccumulator>,
    from: HashMap<Entity, AnimationAccumulator>,
) {
    for (target_entity, values) in from {
        let entry = into.entry(target_entity).or_default();
        if values.translation.is_some() {
            entry.translation = values.translation;
        }
        if values.rotation.is_some() {
            entry.rotation = values.rotation;
        }
        if values.scale.is_some() {
            entry.scale = values.scale;
        }
        if values.morph_weights.is_some() {
            entry.morph_weights = values.morph_weights;
        }
    }
}

fn sample_clip(
    clip: &AnimationClip,
    time: f32,
    node_mapping: &HashMap<usize, Entity>,
    bone_name_mapping: &HashMap<String, Entity>,
) -> HashMap<Entity, AnimationAccumulator> {
    let mut entity_values: HashMap<Entity, AnimationAccumulator> = HashMap::new();

    for channel in clip.channels.iter() {
        let target_entity = if let Some(ref bone_name) = channel.target_bone_name {
            bone_name_mapping.get(bone_name).copied()
        } else {
            None
        }
        .or_else(|| node_mapping.get(&channel.target_node).copied());

        if let Some(target_entity) = target_entity {
            let sampled_value = sample_animation_channel(channel, time);
            if let Some(value) = sampled_value {
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
    if world.core.entity_has_components(entity, MORPH_WEIGHTS)
        && let Some(morph_weights) = world.core.get_morph_weights_mut(entity)
    {
        morph_weights.weights = weights.to_vec();
    }

    let child_entities: Vec<Entity> = world
        .resources
        .transform_state
        .children_cache
        .get(&entity)
        .cloned()
        .unwrap_or_default();

    for child in child_entities {
        apply_morph_weights_to_entity_and_children(world, child, weights);
    }
}
