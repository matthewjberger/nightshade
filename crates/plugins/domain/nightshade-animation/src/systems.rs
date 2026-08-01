pub mod sync;

pub use sync::render_sync_animation_system;

use crate::commands::blend_tree::{collect_blend_tree_clips, evaluate_blend_tree};
use crate::commands::graph::evaluate_animation_graph;
use crate::commands::player::advance_player;
use crate::contract::{
    ActiveClip, AnimationPlayer, AnimationProperty, AnimationValue, BoneRouting,
};
use crate::data::{effective_loop_mode, sample_animation_channel, wrap_time};
use nalgebra_glm::{Quat, Vec3};
use nightshade::ecs::Entity;
use std::collections::HashMap;

pub fn update_animation_players(
    time: nightshade::ecs::system_param::Res<nightshade_platform::contract::time::Time>,
    mut finished_writer: nightshade::ecs::system_param::EventWriter<
        crate::contract::AnimationFinished,
    >,
    mut marker_writer: nightshade::ecs::system_param::EventWriter<
        crate::contract::AnimationMarkerReached,
    >,
    storage: &mut nightshade::ecs::Storage,
) {
    let delta_time = time.delta_time;
    let mut finished: Vec<Entity> = Vec::new();
    let mut markers: Vec<(Entity, String)> = Vec::new();
    let player_key = storage.component_key::<crate::contract::AnimationPlayer>();
    storage.for_each_mut::<(crate::contract::AnimationPlayer,), ()>(|entity, table, idx| {
        let player = &mut table.column_mut(player_key)[idx];
        let was_playing = player.playing;
        let old_time = player.time;
        let looping = player.looping;
        let current_clip = player.current_clip;
        refresh_routing(player);
        advance_player(player, delta_time);
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
        finished_writer.send(crate::contract::AnimationFinished { entity });
    }
    for (entity, name) in markers {
        marker_writer.send(crate::contract::AnimationMarkerReached { entity, name });
    }
}

#[derive(Default)]
struct PoseScratch {
    values: Vec<AnimationAccumulator>,
    held: Vec<bool>,
    other: Vec<AnimationAccumulator>,
    other_held: Vec<bool>,
    allowed: Vec<bool>,
}

impl PoseScratch {
    fn ready(&mut self, bones: usize) {
        self.values.clear();
        self.values
            .resize_with(bones, AnimationAccumulator::default);
        self.held.clear();
        self.held.resize(bones, false);
        self.other.clear();
        self.other.resize_with(bones, AnimationAccumulator::default);
        self.other_held.clear();
        self.other_held.resize(bones, false);
    }

    fn clear_other(&mut self) {
        for (slot, held) in self.other_held.iter_mut().enumerate() {
            if *held {
                self.other[slot] = AnimationAccumulator::default();
                *held = false;
            }
        }
    }
}

pub fn refresh_routing(player: &mut AnimationPlayer) {
    let signature = (
        player.clips.iter().map(|clip| clip.channels.len()).sum(),
        player.node_index_to_entity.len(),
        player.bone_name_to_entity.len(),
    );
    if player.routing.signature == signature && !player.routing.channel_bone.is_empty() {
        return;
    }
    let mut bones: Vec<Entity> = Vec::new();
    let mut slot_of: HashMap<Entity, u32> = HashMap::new();
    let mut channel_bone: Vec<Vec<u32>> = Vec::with_capacity(player.clips.len());
    for clip in &player.clips {
        let mut slots: Vec<u32> = Vec::with_capacity(clip.channels.len());
        for channel in &clip.channels {
            let target = channel
                .target_bone_name
                .as_ref()
                .and_then(|name| player.bone_name_to_entity.get(name).copied())
                .or_else(|| {
                    player
                        .node_index_to_entity
                        .get(&channel.target_node)
                        .copied()
                });
            match target {
                Some(entity) => {
                    let slot = *slot_of.entry(entity).or_insert_with(|| {
                        bones.push(entity);
                        (bones.len() - 1) as u32
                    });
                    slots.push(slot);
                }
                None => slots.push(BoneRouting::NONE),
            }
        }
        channel_bone.push(slots);
    }
    player.routing = BoneRouting {
        bones,
        channel_bone,
        signature,
    };
}

pub fn apply_animations(
    hierarchy: nightshade::ecs::system_param::Res<nightshade::ecs::HierarchyIndex>,
    storage: &mut nightshade::ecs::Storage,
) {
    let mut scratch = PoseScratch::default();
    let mut posed: Vec<(Entity, AnimationAccumulator)> = Vec::new();
    for (_, player) in storage
        .query_ref::<&crate::contract::AnimationPlayer>()
        .iter()
    {
        if !player.playing || player.routing.bones.is_empty() {
            continue;
        }
        if player.routing.bones.first().is_some_and(|bone| {
            storage
                .get::<nightshade_scene::contract::Frozen>(*bone)
                .is_some_and(|frozen| frozen.0)
        }) {
            continue;
        }
        scratch.ready(player.routing.bones.len());

        if player.play_all {
            for index in 0..player.clips.len() {
                sample_clip_into(player, index, player.time, &mut scratch, false);
            }
        } else if let Some(index) = player.current_clip {
            sample_clip_into(player, index, player.time, &mut scratch, false);
        }

        if let Some(index) = player.blend_from_clip
            && index < player.clips.len()
        {
            scratch.clear_other();
            sample_clip_into(player, index, player.blend_from_time, &mut scratch, true);
            for slot in 0..scratch.values.len() {
                if !scratch.held[slot] && !scratch.other_held[slot] {
                    continue;
                }
                let from = scratch.other_held[slot].then(|| scratch.other[slot].clone());
                let blended =
                    blend_accumulators(from.as_ref(), &scratch.values[slot], player.blend_factor);
                scratch.values[slot] = blended;
                scratch.held[slot] = true;
            }
        }

        for layer in &player.layers {
            if !layer.playing || layer.weight <= 0.0 || layer.clip_index >= player.clips.len() {
                continue;
            }
            scratch.clear_other();
            sample_clip_into(player, layer.clip_index, layer.time, &mut scratch, true);
            scratch.allowed.clear();
            if !layer.mask.is_empty() {
                scratch.allowed.resize(player.routing.bones.len(), false);
                for bone in &layer.mask {
                    if let Some(entity) = player.bone_name_to_entity.get(bone)
                        && let Some(slot) =
                            player.routing.bones.iter().position(|held| held == entity)
                    {
                        scratch.allowed[slot] = true;
                    }
                }
            }
            for slot in 0..scratch.other.len() {
                if !scratch.other_held[slot] {
                    continue;
                }
                if !scratch.allowed.is_empty() && !scratch.allowed[slot] {
                    continue;
                }
                let held = scratch.held[slot].then(|| scratch.values[slot].clone());
                scratch.values[slot] =
                    blend_accumulators(held.as_ref(), &scratch.other[slot], layer.weight);
                scratch.held[slot] = true;
            }
        }

        for slot in 0..scratch.values.len() {
            if !scratch.held[slot] {
                continue;
            }
            posed.push((
                player.routing.bones[slot],
                std::mem::take(&mut scratch.values[slot]),
            ));
        }
    }

    for (target_entity, values) in posed {
        if let Some(transform) =
            storage.get_mut::<nightshade_scene::contract::LocalTransform>(target_entity)
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
            apply_morph_weights_to_entity_and_children(
                &hierarchy,
                storage,
                target_entity,
                &weights,
            );
        }
    }
}

fn sample_clip_into(
    player: &AnimationPlayer,
    clip_index: usize,
    time: f32,
    scratch: &mut PoseScratch,
    into_other: bool,
) {
    let Some(clip) = player.clips.get(clip_index) else {
        return;
    };
    let Some(slots) = player.routing.channel_bone.get(clip_index) else {
        return;
    };
    for (index, channel) in clip.channels.iter().enumerate() {
        let Some(&slot) = slots.get(index) else {
            continue;
        };
        if slot == BoneRouting::NONE {
            continue;
        }
        let Some(value) = sample_animation_channel(channel, time) else {
            continue;
        };
        let slot = slot as usize;
        let (entry, held) = if into_other {
            (&mut scratch.other[slot], &mut scratch.other_held[slot])
        } else {
            (&mut scratch.values[slot], &mut scratch.held[slot])
        };
        *held = true;
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

fn apply_morph_weights_to_entity_and_children(
    hierarchy: &nightshade::ecs::HierarchyIndex,
    storage: &mut nightshade::ecs::Storage,
    entity: Entity,
    weights: &[f32],
) {
    let mut pending = vec![entity];
    while let Some(current) = pending.pop() {
        if storage
            .entity_has_components::<(nightshade_render::components::morph::MorphWeights,)>(current)
            && let Some(morph_weights) =
                storage.get_mut::<nightshade_render::components::morph::MorphWeights>(current)
        {
            morph_weights.weights = weights.to_vec();
        }

        pending.extend_from_slice(hierarchy.children(current));
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
    world: &nightshade::ecs::Storage,
    player: &AnimationPlayer,
    entity: Entity,
) -> nalgebra_glm::Mat4 {
    let rest = world
        .get::<nightshade_scene::contract::LocalTransform>(entity)
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
    world: &nightshade::ecs::Storage,
    player: &AnimationPlayer,
    entity: Entity,
) -> nalgebra_glm::Mat4 {
    let local = animated_local_matrix(world, player, entity);
    if let Some(parent) = world
        .get::<nightshade::ecs::ChildOf>(entity)
        .map(|child_of| child_of.0)
    {
        if world.has::<nightshade_render::components::skin::Joint>(parent) {
            return resolve_joint_world(world, player, parent) * local;
        }
        if let Some(parent_world) = world.get::<nightshade_scene::contract::GlobalTransform>(parent)
        {
            return parent_world.0 * local;
        }
    }
    local
}

pub fn resolve_bone_world_transform(
    world: &nightshade::ecs::Storage,
    player_entity: Entity,
    bone_name: &str,
) -> Option<nalgebra_glm::Mat4> {
    let player = world.get::<crate::contract::AnimationPlayer>(player_entity)?;
    let bone = *player.bone_name_to_entity.get(bone_name)?;
    Some(resolve_joint_world(world, player, bone))
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

pub fn apply_root_motion(storage: &mut nightshade::ecs::Storage) {
    let mut updates: Vec<(Entity, Vec3, Vec3, f32)> = Vec::new();
    for (entity, player) in storage
        .query_ref::<&crate::contract::AnimationPlayer>()
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
                storage
                    .get::<nightshade::ecs::ChildOf>(hip)
                    .map(|child_of| child_of.0)
            })
            .map(|parent| nalgebra_glm::mat4_to_mat3(&resolve_joint_world(storage, player, parent)))
            .unwrap_or_else(nalgebra_glm::Mat3::identity);
        let mut world_delta = basis * delta;
        if player.root_motion_planar {
            world_delta.y = 0.0;
        }
        updates.push((entity, world_delta, current, player.time));
    }
    for (entity, world_delta, previous, previous_time) in updates {
        if let Some(player) = storage.get_mut::<crate::contract::AnimationPlayer>(entity) {
            player.root_motion_previous = Some(previous);
            player.root_motion_previous_time = previous_time;
        }
        if world_delta != Vec3::zeros()
            && let Some(transform) =
                storage.get_mut::<nightshade_scene::contract::LocalTransform>(entity)
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
    if player.synchronize_blend {
        let mut weighted = 0.0_f32;
        let mut total = 0.0_f32;
        for (active, duration) in player.active_clips.iter().zip(durations.iter()) {
            if active.weight <= 0.0 {
                continue;
            }
            let Some(duration) = duration.filter(|value| *value > 0.0) else {
                continue;
            };
            weighted += duration * active.weight;
            total += active.weight;
        }
        if total > 0.0 && weighted > 0.0 {
            let average = weighted / total;
            player.blend_phase = (player.blend_phase + delta_time / average).rem_euclid(1.0);
            let phase = player.blend_phase;
            for (active, duration) in player.active_clips.iter_mut().zip(durations) {
                let Some(duration) = duration.filter(|value| *value > 0.0) else {
                    continue;
                };
                active.time = phase * duration;
            }
            return;
        }
    }
    for (active, duration) in player.active_clips.iter_mut().zip(durations) {
        let Some(duration) = duration.filter(|value| *value > 0.0) else {
            continue;
        };
        active.time += delta_time * active.speed;
        let (wrapped, _) = wrap_time(active.time, duration, active.loop_mode);
        active.time = wrapped;
    }
}

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

pub fn evaluate_animation_graphs(
    time: nightshade::ecs::system_param::Res<nightshade_platform::contract::time::Time>,
    storage: &mut nightshade::ecs::Storage,
) {
    let delta_time = time.delta_time;
    let player_key = storage.component_key::<AnimationPlayer>();
    storage.for_each_mut::<(crate::contract::AnimationPlayer,), ()>(|_entity, table, idx| {
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

pub fn evaluate_blend_trees(storage: &mut nightshade::ecs::Storage) {
    let player_key = storage.component_key::<AnimationPlayer>();
    storage.for_each_mut::<(crate::contract::AnimationPlayer,), ()>(|_entity, table, idx| {
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
