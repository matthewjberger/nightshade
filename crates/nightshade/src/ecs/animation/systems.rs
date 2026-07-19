use crate::ecs::animation::components::{
    AnimationClip, AnimationProperty, AnimationValue, sample_animation_channel,
};
use crate::ecs::world::{ANIMATION_PLAYER, MORPH_WEIGHTS, World};
use nalgebra_glm::{Quat, Vec3};
use nightshade_ecs::Entity;
use std::collections::HashMap;

use crate::prelude::*;
pub fn update_animation_players(world: &mut World) {
    let _span = tracing::info_span!("animation").entered();
    let delta_time = world.res::<crate::ecs::time::Time>().delta_time;
    let mut finished: Vec<Entity> = Vec::new();
    let mut markers: Vec<(Entity, String)> = Vec::new();
    let player_key =
        world.ecs.worlds[CORE].register::<crate::ecs::animation::components::AnimationPlayer>();
    world.ecs.worlds[CORE].for_each_mut(ANIMATION_PLAYER, 0, |entity, table, idx| {
        let player = &mut table.column_mut(player_key)[idx];
        let was_playing = player.playing;
        let old_time = player.time;
        let looping = player.looping;
        let current_clip = player.current_clip;
        player.update(delta_time);
        if was_playing && !player.playing {
            finished.push(entity);
        }
        if !was_playing {
            return;
        }
        let new_time = player.time;
        if let Some(index) = current_clip
            && let Some(clip) = player.clips.get(index)
            && !clip.events.is_empty()
        {
            let wrapped = new_time < old_time && looping && clip.duration > 0.0;
            for marker in &clip.events {
                let crossed = if wrapped {
                    marker.time > old_time || marker.time <= new_time
                } else {
                    marker.time > old_time && marker.time <= new_time
                };
                if crossed {
                    markers.push((entity, marker.name.clone()));
                }
            }
        }
    });
    for entity in finished {
        crate::ecs::event::emit_event(
            world,
            crate::ecs::event::Event::AnimationFinished { entity },
        );
    }
    for (entity, name) in markers {
        crate::ecs::event::emit_event(
            world,
            crate::ecs::event::Event::AnimationEvent { entity, name },
        );
    }
}

pub fn apply_animations(world: &mut World) {
    // Joint transforms are applied on the CPU so that joint global transforms
    // reflect the current pose. Consumers that read those transforms on the CPU
    // (skinned shadow casting, attachment sockets, picking) then match the
    // rendered pose instead of the bind pose. Morph weights and non-joint node
    // targets are applied here as well.
    let mut per_player_values: Vec<HashMap<Entity, AnimationAccumulator>> = Vec::new();
    for (_, player) in world
        .query_ref::<&crate::ecs::animation::components::AnimationPlayer>()
        .iter()
    {
        {
            if !player.playing {
                continue;
            }
            let node_mapping = &player.node_index_to_entity;
            let bone_name_mapping = &player.bone_name_to_entity;

            let mut final_values: HashMap<Entity, AnimationAccumulator> = if player.play_all {
                let mut merged = HashMap::new();
                for clip in &player.clips {
                    merge_accumulators(
                        &mut merged,
                        sample_clip(clip, player.time, node_mapping, bone_name_mapping),
                    );
                }
                merged
            } else if let Some(clip) = player.get_current_clip() {
                sample_clip(clip, player.time, node_mapping, bone_name_mapping)
            } else {
                HashMap::new()
            };

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

            for layer in &player.layers {
                if !layer.playing || layer.weight <= 0.0 {
                    continue;
                }
                let Some(clip) = player.clips.get(layer.clip_index) else {
                    continue;
                };
                let layer_values = sample_clip(clip, layer.time, node_mapping, bone_name_mapping);
                let masked: Option<std::collections::HashSet<Entity>> = if layer.mask.is_empty() {
                    None
                } else {
                    Some(
                        layer
                            .mask
                            .iter()
                            .filter_map(|bone| bone_name_mapping.get(bone).copied())
                            .collect(),
                    )
                };
                for (entity, layer_value) in layer_values {
                    if let Some(allowed) = &masked
                        && !allowed.contains(&entity)
                    {
                        continue;
                    }
                    let blended =
                        blend_accumulators(final_values.get(&entity), &layer_value, layer.weight);
                    final_values.insert(entity, blended);
                }
            }

            if !final_values.is_empty() {
                per_player_values.push(final_values);
            }
        }
    }

    for final_values in per_player_values {
        for (target_entity, values) in final_values {
            if let Some(transform) =
                world.get_mut::<crate::ecs::transform::components::LocalTransform>(target_entity)
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
    let mut pending = vec![entity];
    while let Some(current) = pending.pop() {
        if world.ecs.worlds[CORE].entity_has_components(current, MORPH_WEIGHTS)
            && let Some(morph_weights) =
                world.get_mut::<crate::ecs::morph::components::MorphWeights>(current)
        {
            morph_weights.weights = weights.to_vec();
        }

        pending.extend_from_slice(
            world
                .res::<nightshade_ecs::dynamic::HierarchyIndex>()
                .children(current),
        );
    }
}
