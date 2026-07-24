use crate::ecs::animation::blend_tree::{collect_blend_tree_clips, evaluate_blend_tree};
use crate::ecs::animation::components::{
    ActiveClip, AnimationClip, AnimationPlayer, AnimationProperty, AnimationValue,
    effective_loop_mode, sample_animation_channel, wrap_time,
};
use crate::ecs::animation::graph::evaluate_animation_graph;
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
        if player.blend_tree.is_some() || player.graph.is_some() {
            advance_active_clips(player, delta_time);
        } else if !player.play_all {
            sync_legacy_active_clips(player);
        }
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
    // Joint transforms are applied on the CPU so joint global transforms reflect
    // the current pose. Consumers that read those globals on the CPU (the
    // skinned shadow pass, which builds its own palette from
    // `render_skinning.bone_transforms`, plus attachment sockets and picking)
    // then match the animated pose. The GPU pose pipeline recomputes the same
    // pose and overwrites the main pass's palette, so the render stays GPU driven
    // while these CPU consumers keep working.
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

fn compose_trs(translation: Vec3, rotation: Quat, scale: Vec3) -> nalgebra_glm::Mat4 {
    let mut matrix = nalgebra_glm::quat_to_mat4(&rotation.normalize());
    for row in 0..3 {
        matrix[(row, 0)] *= scale.x;
        matrix[(row, 1)] *= scale.y;
        matrix[(row, 2)] *= scale.z;
    }
    matrix[(0, 3)] = translation.x;
    matrix[(1, 3)] = translation.y;
    matrix[(2, 3)] = translation.z;
    matrix
}

fn animated_local_matrix(
    world: &World,
    player: &AnimationPlayer,
    entity: Entity,
) -> nalgebra_glm::Mat4 {
    let rest = world
        .get::<crate::ecs::transform::components::LocalTransform>(entity)
        .copied()
        .unwrap_or_default();
    let mut translation = rest.translation;
    let mut rotation = rest.rotation;
    let mut scale = rest.scale;
    let mut running_weight = 0.0_f32;
    for active in &player.active_clips {
        if active.weight <= 0.0 {
            continue;
        }
        let Some(clip) = player.clips.get(active.clip_index) else {
            continue;
        };
        let mut sampled_translation = rest.translation;
        let mut sampled_rotation = rest.rotation;
        let mut sampled_scale = rest.scale;
        for channel in &clip.channels {
            if player.resolve_target_entity(channel) != Some(entity) {
                continue;
            }
            if let Some(value) = sample_animation_channel(channel, active.time) {
                match (channel.target_property, value) {
                    (AnimationProperty::Translation, AnimationValue::Vec3(vector)) => {
                        sampled_translation = vector;
                    }
                    (AnimationProperty::Rotation, AnimationValue::Quat(quaternion)) => {
                        sampled_rotation = quaternion;
                    }
                    (AnimationProperty::Scale, AnimationValue::Vec3(vector)) => {
                        sampled_scale = vector;
                    }
                    _ => {}
                }
            }
        }
        let factor = active.weight / (running_weight + active.weight);
        translation = translation.lerp(&sampled_translation, factor);
        let dot = rotation.dot(&sampled_rotation);
        let adjusted = if dot < 0.0 {
            -sampled_rotation
        } else {
            sampled_rotation
        };
        rotation = nalgebra_glm::quat_slerp(&rotation.normalize(), &adjusted.normalize(), factor);
        scale = scale.lerp(&sampled_scale, factor);
        running_weight += active.weight;
    }
    compose_trs(translation, rotation, scale)
}

fn resolve_joint_world(
    world: &World,
    player: &AnimationPlayer,
    entity: Entity,
) -> nalgebra_glm::Mat4 {
    let local = animated_local_matrix(world, player, entity);
    if let Some(parent) = world
        .get::<nightshade_ecs::dynamic::ChildOf>(entity)
        .map(|child_of| child_of.0)
    {
        if world.has::<crate::ecs::skin::components::Joint>(parent) {
            return resolve_joint_world(world, player, parent) * local;
        }
        if let Some(parent_world) =
            world.get::<crate::ecs::transform::components::GlobalTransform>(parent)
        {
            return parent_world.0 * local;
        }
    }
    local
}

/// Resolves a single bone's current animated world transform on the CPU by
/// blending the player's active clips down that bone's chain, for attachment
/// sockets and foot placement without a GPU readback.
pub fn resolve_bone_world_transform(
    world: &World,
    player_entity: Entity,
    bone_name: &str,
) -> Option<nalgebra_glm::Mat4> {
    let player = world.get::<crate::ecs::animation::components::AnimationPlayer>(player_entity)?;
    let bone = *player.bone_name_to_entity.get(bone_name)?;
    Some(resolve_joint_world(world, player, bone))
}

/// Steers each leg's two-bone IK chain so the foot plants on the ground it stands
/// over, never below the animated pose. Raycasts the physics world from each
/// foot's animated position.
type FootProbe = (usize, String, String, String, f32, f32);

fn entity_forward(world: &World, entity: Entity) -> Vec3 {
    world
        .get::<crate::ecs::transform::components::GlobalTransform>(entity)
        .map(|transform| {
            let forward = Vec3::new(transform.0[(0, 2)], 0.0, transform.0[(2, 2)]);
            if forward.magnitude() > 1.0e-4 {
                forward.normalize()
            } else {
                Vec3::z()
            }
        })
        .unwrap_or_else(Vec3::z)
}

/// Places a pole so the two-bone solve bends in the leg's sagittal plane. The
/// bend axis stays horizontal (`cross(down_the_leg, forward)`), so the knee can
/// never splay sideways the way the animated-plane fallback does when the leg
/// passes through straight; the sign comes from which side the animated knee sits
/// on, preserving the natural fore/aft bend of each limb.
fn sagittal_pole(hip: Vec3, knee: Vec3, foot: Vec3, forward: Vec3) -> Vec3 {
    let chord_mid = (hip + foot) * 0.5;
    let sign = if (knee - chord_mid).dot(&forward) >= 0.0 {
        1.0
    } else {
        -1.0
    };
    let span = (foot - hip).magnitude().max(0.01);
    knee + forward * (sign * span)
}

/// An optional ground-query resolver for foot IK: casts a ray and returns
/// the ground hit point, if any. The physics plugin installs one; without it
/// foot IK leaves feet at their animated positions. Kept opaque so the
/// animation domain does not depend on the physics plugin.
type FootIkGroundResolver =
    Box<dyn Fn(&World, Vec3, Vec3, f32, Option<Entity>) -> Option<Vec3> + Send + Sync>;

#[derive(Default)]
pub struct FootIkGroundQuery {
    pub resolver: Option<FootIkGroundResolver>,
}

pub fn foot_ik_system(world: &mut World) {
    let mut player_feet: Vec<(Entity, Vec<FootProbe>)> = Vec::new();
    for (entity, player) in world
        .query_ref::<&crate::ecs::animation::components::AnimationPlayer>()
        .iter()
    {
        if !player.foot_ik_enabled || player.foot_ik.is_empty() {
            continue;
        }
        let feet = player
            .foot_ik
            .iter()
            .filter_map(|foot| {
                let chain = player.ik_chains.get(foot.ik_chain_index)?;
                Some((
                    foot.ik_chain_index,
                    chain.root_bone.clone(),
                    chain.mid_bone.clone(),
                    foot.tip_bone.clone(),
                    foot.ray_length,
                    foot.foot_height,
                ))
            })
            .collect();
        player_feet.push((entity, feet));
    }

    let ground_query = world.res_mut::<FootIkGroundQuery>().resolver.take();
    let mut updates: Vec<(Entity, usize, Vec3, Option<Vec3>, bool)> = Vec::new();
    for (entity, feet) in player_feet {
        let forward = entity_forward(world, entity);
        for (chain_index, root_bone, mid_bone, tip_bone, ray_length, foot_height) in feet {
            let Some(foot_world) = resolve_bone_world_transform(world, entity, &tip_bone) else {
                continue;
            };
            let foot_position =
                Vec3::new(foot_world[(0, 3)], foot_world[(1, 3)], foot_world[(2, 3)]);
            let bone_position = |bone: &str| {
                resolve_bone_world_transform(world, entity, bone)
                    .map(|matrix| Vec3::new(matrix[(0, 3)], matrix[(1, 3)], matrix[(2, 3)]))
            };
            let pole = match (bone_position(&root_bone), bone_position(&mid_bone)) {
                (Some(hip), Some(knee)) => Some(sagittal_pole(hip, knee, foot_position, forward)),
                _ => None,
            };
            let origin = foot_position + Vec3::new(0.0, ray_length, 0.0);
            let hit_point = ground_query.as_ref().and_then(|resolve| {
                resolve(
                    world,
                    origin,
                    Vec3::new(0.0, -1.0, 0.0),
                    ray_length * 2.0,
                    Some(entity),
                )
            });
            let (target, enabled) = match hit_point {
                Some(point) => {
                    let desired_y = (point.y + foot_height).max(foot_position.y);
                    (Vec3::new(foot_position.x, desired_y, foot_position.z), true)
                }
                None => (foot_position, false),
            };
            updates.push((entity, chain_index, target, pole, enabled));
        }
    }
    world.res_mut::<FootIkGroundQuery>().resolver = ground_query;

    for (entity, chain_index, target, pole, enabled) in updates {
        if let Some(player) =
            world.get_mut::<crate::ecs::animation::components::AnimationPlayer>(entity)
            && let Some(chain) = player.ik_chains.get_mut(chain_index)
        {
            chain.target = target;
            chain.pole = pole;
            chain.enabled = enabled;
        }
    }
}

fn extract_root_translation(player: &AnimationPlayer, bone_name: &str) -> Option<Vec3> {
    let bone_entity = *player.bone_name_to_entity.get(bone_name)?;
    let mut accumulated = Vec3::zeros();
    let mut total_weight = 0.0_f32;
    for active in &player.active_clips {
        if active.weight <= 0.0 {
            continue;
        }
        let Some(clip) = player.clips.get(active.clip_index) else {
            continue;
        };
        for channel in &clip.channels {
            if channel.target_property != AnimationProperty::Translation {
                continue;
            }
            if player.resolve_target_entity(channel) != Some(bone_entity) {
                continue;
            }
            if let Some(AnimationValue::Vec3(translation)) =
                sample_animation_channel(channel, active.time)
            {
                accumulated += translation * active.weight;
                total_weight += active.weight;
            }
        }
    }
    if total_weight > 0.0 {
        Some(accumulated / total_weight)
    } else {
        None
    }
}

/// Drives the entity transform from the root bone's weighted translation, the
/// travel the GPU pose locks out of the rendered hips. Runs after the pose is
/// resolved; loop wraps zero the delta so the character does not snap backward.
pub fn apply_root_motion(world: &mut World) {
    let mut updates: Vec<(Entity, Vec3, Vec3, f32)> = Vec::new();
    for (entity, player) in world
        .query_ref::<&crate::ecs::animation::components::AnimationPlayer>()
        .iter()
    {
        if !player.root_motion {
            continue;
        }
        let Some(bone_name) = player.root_motion_bone.clone() else {
            continue;
        };
        let Some(current) = extract_root_translation(player, &bone_name) else {
            continue;
        };
        let delta = match player.root_motion_previous {
            Some(previous) if player.time >= player.root_motion_previous_time => current - previous,
            _ => Vec3::zeros(),
        };
        let basis = player
            .bone_name_to_entity
            .get(&bone_name)
            .copied()
            .and_then(|hip| {
                world
                    .get::<nightshade_ecs::dynamic::ChildOf>(hip)
                    .map(|child_of| child_of.0)
            })
            .map(|parent| nalgebra_glm::mat4_to_mat3(&resolve_joint_world(world, player, parent)))
            .unwrap_or_else(nalgebra_glm::Mat3::identity);
        let mut world_delta = basis * delta;
        if player.root_motion_planar {
            world_delta.y = 0.0;
        }
        updates.push((entity, world_delta, current, player.time));
    }
    for (entity, world_delta, previous, previous_time) in updates {
        if let Some(player) =
            world.get_mut::<crate::ecs::animation::components::AnimationPlayer>(entity)
        {
            player.root_motion_previous = Some(previous);
            player.root_motion_previous_time = previous_time;
        }
        if world_delta != Vec3::zeros()
            && let Some(transform) =
                world.get_mut::<crate::ecs::transform::components::LocalTransform>(entity)
        {
            transform.translation += world_delta;
        }
    }
}

fn sync_legacy_active_clips(player: &mut AnimationPlayer) {
    let loop_mode = effective_loop_mode(player.loop_mode, player.looping);
    let mut active: Vec<ActiveClip> = Vec::new();
    if let Some(current) = player.current_clip {
        let blending = player.blend_from_clip.is_some();
        let weight = if blending { player.blend_factor } else { 1.0 };
        active.push(ActiveClip {
            clip_index: current,
            time: player.time,
            weight,
            target_weight: weight,
            fade_rate: 0.0,
            speed: player.speed,
            loop_mode,
            dominant: true,
        });
        if let Some(from) = player.blend_from_clip {
            let from_weight = 1.0 - player.blend_factor;
            active.push(ActiveClip {
                clip_index: from,
                time: player.blend_from_time,
                weight: from_weight,
                target_weight: from_weight,
                fade_rate: 0.0,
                speed: player.speed,
                loop_mode,
                dominant: false,
            });
        }
    }
    player.active_clips = active;
}

fn advance_active_clips(player: &mut AnimationPlayer, delta_time: f32) {
    if player.active_clips.is_empty() {
        return;
    }
    let durations: Vec<Option<f32>> = player
        .active_clips
        .iter()
        .map(|active| {
            player
                .clips
                .get(active.clip_index)
                .map(|clip| clip.duration)
        })
        .collect();
    for (active, duration) in player.active_clips.iter_mut().zip(durations) {
        let Some(duration) = duration.filter(|value| *value > 0.0) else {
            continue;
        };
        active.time += delta_time * active.speed;
        let (wrapped, _) = wrap_time(active.time, duration, active.loop_mode);
        active.time = wrapped;
    }
}

/// Rebuilds `active_clips` from a weighted clip set, preserving the playback time
/// of any clip already active so motion stays continuous as weights shift, and
/// marking the highest-weight clip dominant for the legacy playback API.
pub fn set_blend_pose(player: &mut AnimationPlayer, all_clips: &[usize], weights: &[(usize, f32)]) {
    let loop_mode = player.loop_mode;
    let mut weight_map: HashMap<usize, f32> = HashMap::new();
    for (clip_index, weight) in weights {
        *weight_map.entry(*clip_index).or_insert(0.0) += *weight;
    }
    let mut previous_times: HashMap<usize, f32> = HashMap::new();
    for active in &player.active_clips {
        previous_times.insert(active.clip_index, active.time);
    }
    let mut new_active: Vec<ActiveClip> = Vec::with_capacity(all_clips.len());
    for clip_index in all_clips {
        let weight = weight_map.get(clip_index).copied().unwrap_or(0.0);
        let time = previous_times.get(clip_index).copied().unwrap_or(0.0);
        new_active.push(ActiveClip {
            clip_index: *clip_index,
            time,
            weight,
            target_weight: weight,
            fade_rate: 0.0,
            speed: 1.0,
            loop_mode,
            dominant: false,
        });
    }
    let dominant = new_active
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| {
            a.weight
                .partial_cmp(&b.weight)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(index, _)| index);
    if let Some(dominant) = dominant {
        new_active[dominant].dominant = true;
        player.current_clip = Some(new_active[dominant].clip_index);
        player.time = new_active[dominant].time;
    }
    player.active_clips = new_active;
    player.playing = true;
}

/// Interprets each player's state graph into its weighted clip set. Runs before
/// blend-tree evaluation; a graph supersedes a standalone blend tree.
pub fn evaluate_animation_graphs(world: &mut World) {
    let delta_time = world.res::<crate::ecs::time::Time>().delta_time;
    let player_key = world.ecs.worlds[CORE].register::<AnimationPlayer>();
    world.ecs.worlds[CORE].for_each_mut(ANIMATION_PLAYER, 0, |_entity, table, idx| {
        let player = &mut table.column_mut(player_key)[idx];
        let Some(mut graph) = player.graph.take() else {
            return;
        };
        let mut weights: Vec<(usize, f32)> = Vec::new();
        let mut all_clips: Vec<usize> = Vec::new();
        evaluate_animation_graph(
            &mut graph,
            &player.parameters,
            delta_time,
            &mut weights,
            &mut all_clips,
        );
        player.graph = Some(graph);
        set_blend_pose(player, &all_clips, &weights);
    });
}

/// Interprets each player's blend tree into its weighted clip set, skipping any
/// player driven by a state graph.
pub fn evaluate_blend_trees(world: &mut World) {
    let player_key = world.ecs.worlds[CORE].register::<AnimationPlayer>();
    world.ecs.worlds[CORE].for_each_mut(ANIMATION_PLAYER, 0, |_entity, table, idx| {
        let player = &mut table.column_mut(player_key)[idx];
        if player.graph.is_some() {
            return;
        }
        let Some(tree) = player.blend_tree.clone() else {
            return;
        };
        let mut clips: Vec<usize> = Vec::new();
        collect_blend_tree_clips(&tree, &mut clips);
        let mut weights: Vec<(usize, f32)> = Vec::new();
        evaluate_blend_tree(&tree, &player.parameters, 1.0, &mut weights);
        set_blend_pose(player, &clips, &weights);
    });
}
