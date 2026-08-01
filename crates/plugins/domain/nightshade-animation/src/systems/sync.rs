use crate::contract::{
    AnimationChannel, AnimationClip, AnimationInterpolation, AnimationLayer, AnimationPlayer,
    AnimationProperty, AnimationSamplerOutput,
};
use crate::resources::{ChannelCache, FrozenJoints};
use nightshade::ecs::Entity;
use nightshade::ecs::system_param::{Res, ResMut};
use nightshade_render::data::config::{SkinnedChannelData, SkinnedJointData, SkinnedSkeletonData};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

const PROPERTY_TRANSLATION: u32 = 0;
const PROPERTY_ROTATION: u32 = 1;
const PROPERTY_SCALE: u32 = 2;

pub fn render_sync_animation_system(
    mut sync: ResMut<nightshade_render::resources::render_sync::RenderSceneSync>,
    mut renderer_state: ResMut<nightshade_render::data::config::RendererState>,
    time: Res<nightshade_platform::contract::time::Time>,
    mut channels: ResMut<ChannelCache>,
    mut frozen: ResMut<FrozenJoints>,
    storage: &mut nightshade::ecs::Storage,
) {
    let cache = &mut channels.0;
    let since = sync.animation_cursor;
    let players_ticked = storage
        .query_entities_changed_since::<(crate::contract::AnimationPlayer,)>(since)
        .next()
        .is_some();
    let skins_ticked = storage
        .query_entities_changed_since::<(nightshade_render::components::skin::Skin,)>(since)
        .next()
        .is_some();
    sync.animation_cursor = sync.data_cursor;

    let mut pose_runtime = std::mem::take(&mut renderer_state.render_animation.pose_runtime);
    pose_runtime.clear();
    let mut ik_runtime = std::mem::take(&mut renderer_state.render_animation.ik_runtime);
    ik_runtime.clear();
    let mut ik_multi_runtime =
        std::mem::take(&mut renderer_state.render_animation.ik_multi_runtime);
    ik_multi_runtime.clear();
    let mut aim_runtime = std::mem::take(&mut renderer_state.render_animation.aim_runtime);
    aim_runtime.clear();
    let delta_time = time.delta_time;
    let mut spring_runtime = std::mem::take(&mut renderer_state.render_animation.spring_runtime);
    spring_runtime.clear();
    let mut player_clips = std::mem::take(&mut sync.animation_player_clips_scratch);
    player_clips.clear();

    for (player_entity, player) in storage.query_ref::<&AnimationPlayer>().iter() {
        if !player.playing || player.play_all || player.active_clips.is_empty() {
            continue;
        }
        let mut clip_hasher = std::collections::hash_map::DefaultHasher::new();
        for (active_index, active) in player.active_clips.iter().enumerate() {
            pose_runtime.insert((player_entity, active_index), [active.time, active.weight]);
            active.clip_index.hash(&mut clip_hasher);
        }
        for (chain_index, chain) in player.ik_chains.iter().enumerate() {
            let pole = chain.pole.unwrap_or_else(nalgebra_glm::Vec3::zeros);
            ik_runtime.insert(
                (player_entity, chain_index),
                [
                    chain.target.x,
                    chain.target.y,
                    chain.target.z,
                    pole.x,
                    pole.y,
                    pole.z,
                    chain.weight,
                    if chain.enabled { 1.0 } else { 0.0 },
                ],
            );
        }
        for (chain_index, chain) in player.ik_chains_multi.iter().enumerate() {
            ik_multi_runtime.insert(
                (player_entity, chain_index),
                [
                    chain.target.x,
                    chain.target.y,
                    chain.target.z,
                    chain.weight,
                    if chain.enabled { 1.0 } else { 0.0 },
                    chain.max_angle,
                    chain.iterations as f32,
                    0.0,
                ],
            );
        }
        for (constraint_index, constraint) in player.aim_constraints.iter().enumerate() {
            aim_runtime.insert(
                (player_entity, constraint_index),
                [
                    constraint.target.x,
                    constraint.target.y,
                    constraint.target.z,
                    constraint.weight,
                    if constraint.enabled { 1.0 } else { 0.0 },
                    constraint.cone_angle,
                    0.0,
                    0.0,
                ],
            );
        }
        for (chain_index, chain) in player.spring_chains.iter().enumerate() {
            spring_runtime.insert(
                (player_entity, chain_index),
                [
                    chain.gravity.x,
                    chain.gravity.y,
                    chain.gravity.z,
                    chain.stiffness,
                    chain.damping,
                    if chain.enabled { 1.0 } else { 0.0 },
                    delta_time,
                    0.0,
                ],
            );
        }
        player_clips.insert(
            player_entity,
            (
                player.active_clips.len() as i64,
                clip_hasher.finish() as i64,
            ),
        );
    }

    let skinned_generation = renderer_state.skinned_generation;
    let gate_fired = {
        let gate_fired = !sync.animation_initialized
            || players_ticked
            || skins_ticked
            || sync.animation_skinned_generation_seen != skinned_generation
            || player_clips != sync.animation_player_clips;
        sync.animation_initialized = true;
        sync.animation_skinned_generation_seen = skinned_generation;
        gate_fired
    };

    let skeletons = if gate_fired {
        let joint_to_player = collect_joint_to_player(storage);
        let mut skin_entities: Vec<Entity> = storage
            .query_entities::<(nightshade_render::components::skin::Skin,)>()
            .collect();
        skin_entities.sort_by_key(|entity| (entity.id, entity.generation));
        let (signature, clips) = signatures(storage, &skin_entities, &joint_to_player);
        let (rebuild, refresh, mut cached) = {
            let snapshot = &mut renderer_state.render_animation;
            let rebuild = signature != snapshot.signature;
            let refresh = !rebuild && clips != snapshot.clip_signature;
            snapshot.signature = signature;
            snapshot.clip_signature = clips;
            (rebuild, refresh, std::mem::take(&mut snapshot.skeletons))
        };
        if rebuild {
            let built = build_skeletons(storage, &skin_entities, &joint_to_player, cache);
            if renderer_state.poses_on_gpu {
                freeze_joints(storage, &built, &mut frozen.0);
            }
            built
        } else {
            if refresh {
                refresh_clip_channels(storage, &mut cached, cache);
            }
            cached
        }
    } else {
        std::mem::take(&mut renderer_state.render_animation.skeletons)
    };

    {
        sync.animation_player_clips_scratch =
            std::mem::replace(&mut sync.animation_player_clips, player_clips);
    }

    let mut layer_runtime = std::mem::take(&mut renderer_state.render_animation.layer_runtime);
    layer_runtime.clear();
    let mut armature_roots = std::mem::take(&mut renderer_state.render_animation.armature_roots);
    armature_roots.clear();
    for skeleton in &skeletons {
        let root = skeleton
            .armature_parent
            .and_then(|parent| storage.get::<nightshade_scene::contract::GlobalTransform>(parent))
            .map(|global| global.0)
            .unwrap_or_else(nalgebra_glm::Mat4::identity);
        armature_roots.insert(skeleton.skin_entity, root);
        if let Some(player) =
            storage.get::<crate::contract::AnimationPlayer>(skeleton.player_entity)
        {
            for layer_index in 0..skeleton.layer_count as usize {
                if let Some(layer) = player.layers.get(layer_index) {
                    layer_runtime.insert(
                        (skeleton.player_entity, layer_index),
                        [layer.time, layer.weight],
                    );
                }
            }
        }
    }

    let snapshot = &mut renderer_state.render_animation;
    snapshot.skeletons = skeletons;
    snapshot.pose_runtime = pose_runtime;
    snapshot.layer_runtime = layer_runtime;
    snapshot.ik_runtime = ik_runtime;
    snapshot.ik_multi_runtime = ik_multi_runtime;
    snapshot.aim_runtime = aim_runtime;
    snapshot.spring_runtime = spring_runtime;
    snapshot.armature_roots = armature_roots;
}

fn freeze_joints(
    storage: &mut nightshade::ecs::Storage,
    skeletons: &[SkinnedSkeletonData],
    frozen: &mut Vec<Entity>,
) {
    for joint in frozen.drain(..) {
        if let Some(held) = storage.get_mut::<nightshade_scene::contract::Frozen>(joint) {
            held.0 = false;
        }
    }
    let mut joints: Vec<Entity> = Vec::new();
    for skeleton in skeletons {
        if let Some(skin) =
            storage.get::<nightshade_render::components::skin::Skin>(skeleton.skin_entity)
        {
            joints.extend(skin.joints.iter().copied());
        }
    }
    for joint in joints {
        match storage.get_mut::<nightshade_scene::contract::Frozen>(joint) {
            Some(held) => held.0 = true,
            None => storage.set(joint, nightshade_scene::contract::Frozen(true)),
        }
        frozen.push(joint);
    }
}

fn collect_joint_to_player(world: &nightshade::ecs::Storage) -> HashMap<Entity, Entity> {
    let mut joint_to_player: HashMap<Entity, Entity> = HashMap::new();
    for (player_entity, player) in world.query_ref::<&AnimationPlayer>().iter() {
        if !player.playing || player.play_all {
            continue;
        }
        for active in &player.active_clips {
            let Some(clip) = player.clips.get(active.clip_index) else {
                continue;
            };
            for channel in &clip.channels {
                if gpu_property(channel.target_property).is_some()
                    && let Some(target) = player.resolve_target_entity(channel)
                {
                    joint_to_player.entry(target).or_insert(player_entity);
                }
            }
        }
    }
    joint_to_player
}

fn resolve_ik_chains(
    player: &AnimationPlayer,
    local_index_of: &HashMap<Entity, usize>,
) -> Vec<nightshade_render::data::config::SkinnedIkChainData> {
    player
        .ik_chains
        .iter()
        .enumerate()
        .filter_map(|(chain_index, chain)| {
            let resolve = |name: &str| {
                player
                    .bone_name_to_entity
                    .get(name)
                    .and_then(|joint| local_index_of.get(joint).copied())
            };
            Some(nightshade_render::data::config::SkinnedIkChainData {
                chain_index: chain_index as u32,
                root_local: resolve(&chain.root_bone)? as u32,
                mid_local: resolve(&chain.mid_bone)? as u32,
                tip_local: resolve(&chain.tip_bone)? as u32,
                has_pole: chain.pole.is_some(),
            })
        })
        .collect()
}

fn channel_key(
    player_entity: Entity,
    clip: &AnimationClip,
    clip_index: usize,
    channel_index: usize,
) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    if clip.source == 0 {
        player_entity.id.hash(&mut hasher);
        player_entity.generation.hash(&mut hasher);
        clip_index.hash(&mut hasher);
    }
    clip.source.hash(&mut hasher);
    channel_index.hash(&mut hasher);
    hasher.finish()
}

fn build_active_clip_channels(
    player: &AnimationPlayer,
    player_entity: Entity,
    cache: &mut HashMap<u64, SkinnedChannelData>,
) -> Vec<HashMap<Entity, Vec<SkinnedChannelData>>> {
    player
        .active_clips
        .iter()
        .map(|active| {
            let mut joint_channels: HashMap<Entity, Vec<SkinnedChannelData>> = HashMap::new();
            if let Some(clip) = player.clips.get(active.clip_index) {
                for (channel_index, channel) in clip.channels.iter().enumerate() {
                    let key = channel_key(player_entity, clip, active.clip_index, channel_index);
                    if let Some(owned) = build_channel(channel, key, clip.source != 0, cache)
                        && let Some(target) = player.resolve_target_entity(channel)
                    {
                        joint_channels.entry(target).or_default().push(owned);
                    }
                }
            }
            joint_channels
        })
        .collect()
}

fn build_skeletons(
    world: &nightshade::ecs::Storage,
    skin_entities: &[Entity],
    joint_to_player: &HashMap<Entity, Entity>,
    cache: &mut HashMap<u64, SkinnedChannelData>,
) -> Vec<SkinnedSkeletonData> {
    let mut skeletons: Vec<SkinnedSkeletonData> = Vec::new();
    for &skin_entity in skin_entities {
        let Some(skin) = world.get::<nightshade_render::components::skin::Skin>(skin_entity) else {
            continue;
        };
        let Some(&player_entity) = skin
            .joints
            .iter()
            .find_map(|joint| joint_to_player.get(joint))
        else {
            continue;
        };

        let local_index_of: HashMap<Entity, usize> = skin
            .joints
            .iter()
            .enumerate()
            .map(|(local, joint)| (*joint, local))
            .collect();

        let mut order: Vec<usize> = (0..skin.joints.len()).collect();
        order.sort_by_key(|&local| joint_depth(world, &skin.joints, &local_index_of, local));

        let player = world.get::<crate::contract::AnimationPlayer>(player_entity);
        let pose_joint_channels = player
            .map(|player| build_active_clip_channels(player, player_entity, cache))
            .unwrap_or_default();
        let active_count = pose_joint_channels.len() as u32;

        let layer_joint_channels: Vec<HashMap<Entity, Vec<SkinnedChannelData>>> = player
            .map(|player| {
                player
                    .layers
                    .iter()
                    .map(|layer| build_layer_joint_channels(player, player_entity, layer, cache))
                    .collect()
            })
            .unwrap_or_default();
        let reference_joint_channels: Vec<HashMap<Entity, Vec<SkinnedChannelData>>> = player
            .map(|player| {
                player
                    .layers
                    .iter()
                    .map(|layer| {
                        build_reference_joint_channels(player, player_entity, layer, cache)
                    })
                    .collect()
            })
            .unwrap_or_default();
        let layer_modes: Vec<u32> = player
            .map(|player| {
                player
                    .layers
                    .iter()
                    .map(|layer| match layer.blend_mode {
                        crate::contract::LayerBlendMode::Additive => 1,
                        crate::contract::LayerBlendMode::Override => 0,
                    })
                    .collect()
            })
            .unwrap_or_default();
        let layer_count = layer_joint_channels.len() as u32;

        let root_motion_local_index = player.and_then(|player| {
            if !player.root_motion {
                return None;
            }
            let bone_name = player.root_motion_bone.as_ref()?;
            let joint = player.bone_name_to_entity.get(bone_name)?;
            local_index_of.get(joint).map(|index| *index as u32)
        });

        let ik_chains: Vec<nightshade_render::data::config::SkinnedIkChainData> = player
            .map(|player| resolve_ik_chains(player, &local_index_of))
            .unwrap_or_default();
        let ik_multi_chains: Vec<nightshade_render::data::config::SkinnedMultiIkChainData> = player
            .map(|player| resolve_multi_ik_chains(player, &local_index_of))
            .unwrap_or_default();
        let aim_constraints: Vec<nightshade_render::data::config::SkinnedAimConstraintData> =
            player
                .map(|player| resolve_aim_constraints(player, &local_index_of))
                .unwrap_or_default();
        let spring_chains: Vec<nightshade_render::data::config::SkinnedSpringChainData> = player
            .map(|player| resolve_spring_chains(world, player, &local_index_of))
            .unwrap_or_default();

        let mut joints_ordered: Vec<SkinnedJointData> = Vec::with_capacity(order.len());
        for &local in &order {
            let joint = skin.joints[local];
            let rest = world
                .get::<nightshade_scene::contract::LocalTransform>(joint)
                .copied()
                .unwrap_or_default();
            let parent_local = world
                .get::<nightshade::ecs::ChildOf>(joint)
                .map(|child_of| child_of.0)
                .and_then(|parent| local_index_of.get(&parent).copied());
            let pose_channels: Vec<Vec<SkinnedChannelData>> = pose_joint_channels
                .iter()
                .map(|channels| channels.get(&joint).cloned().unwrap_or_default())
                .collect();
            let layer_channels: Vec<Vec<SkinnedChannelData>> = layer_joint_channels
                .iter()
                .map(|channels| channels.get(&joint).cloned().unwrap_or_default())
                .collect();
            let reference_channels: Vec<Vec<SkinnedChannelData>> = reference_joint_channels
                .iter()
                .map(|channels| channels.get(&joint).cloned().unwrap_or_default())
                .collect();

            joints_ordered.push(SkinnedJointData {
                local_index: local as u32,
                parent_local: parent_local.map(|parent| parent as u32),
                rest_translation: [rest.translation.x, rest.translation.y, rest.translation.z],
                rest_rotation: [
                    rest.rotation.coords.x,
                    rest.rotation.coords.y,
                    rest.rotation.coords.z,
                    rest.rotation.coords.w,
                ],
                rest_scale: [rest.scale.x, rest.scale.y, rest.scale.z],
                pose_channels,
                layer_channels,
                reference_channels,
            });
        }

        skeletons.push(SkinnedSkeletonData {
            skin_entity,
            player_entity,
            armature_parent: skin_armature_parent(world, skin_entity),
            joint_count: skin.joints.len() as u32,
            active_count,
            layer_count,
            layer_modes,
            root_motion_local_index,
            ik_chains,
            ik_multi_chains,
            aim_constraints,
            spring_chains,
            joints_ordered,
        });
    }
    skeletons
}

fn gpu_property(property: AnimationProperty) -> Option<u32> {
    match property {
        AnimationProperty::Translation => Some(PROPERTY_TRANSLATION),
        AnimationProperty::Rotation => Some(PROPERTY_ROTATION),
        AnimationProperty::Scale => Some(PROPERTY_SCALE),
        AnimationProperty::MorphWeights => None,
    }
}

fn build_channel(
    channel: &AnimationChannel,
    key: u64,
    shared: bool,
    cache: &mut HashMap<u64, SkinnedChannelData>,
) -> Option<SkinnedChannelData> {
    if shared && let Some(found) = cache.get(&key) {
        return Some(found.clone());
    }
    let property = gpu_property(channel.target_property)?;
    let sampler = &channel.sampler;
    let interpolation = match sampler.interpolation {
        AnimationInterpolation::Linear => 0,
        AnimationInterpolation::Step => 1,
        AnimationInterpolation::CubicSpline => 2,
    };
    let mut values: Vec<[f32; 4]> = Vec::new();
    let stride = match &sampler.output {
        AnimationSamplerOutput::Vec3(samples) => {
            for value in samples {
                values.push([value.x, value.y, value.z, 0.0]);
            }
            1
        }
        AnimationSamplerOutput::Quat(samples) => {
            for value in samples {
                let coords = value.coords;
                values.push([coords.x, coords.y, coords.z, coords.w]);
            }
            1
        }
        AnimationSamplerOutput::CubicSplineVec3 {
            values: samples,
            in_tangents,
            out_tangents,
        } => {
            for index in 0..samples.len() {
                let in_tangent = in_tangents[index];
                let value = samples[index];
                let out_tangent = out_tangents[index];
                values.push([in_tangent.x, in_tangent.y, in_tangent.z, 0.0]);
                values.push([value.x, value.y, value.z, 0.0]);
                values.push([out_tangent.x, out_tangent.y, out_tangent.z, 0.0]);
            }
            3
        }
        AnimationSamplerOutput::CubicSplineQuat {
            values: samples,
            in_tangents,
            out_tangents,
        } => {
            for index in 0..samples.len() {
                let in_tangent = in_tangents[index].coords;
                let value = samples[index].coords;
                let out_tangent = out_tangents[index].coords;
                values.push([in_tangent.x, in_tangent.y, in_tangent.z, in_tangent.w]);
                values.push([value.x, value.y, value.z, value.w]);
                values.push([out_tangent.x, out_tangent.y, out_tangent.z, out_tangent.w]);
            }
            3
        }
        AnimationSamplerOutput::Weights(_) | AnimationSamplerOutput::CubicSplineWeights { .. } => {
            return None;
        }
    };
    let built = SkinnedChannelData {
        key,
        property,
        interpolation,
        input: sampler.input.clone().into(),
        values: values.into(),
        stride,
    };
    if shared {
        cache.insert(key, built.clone());
    }
    Some(built)
}

fn build_layer_joint_channels(
    player: &AnimationPlayer,
    player_entity: Entity,
    layer: &AnimationLayer,
    cache: &mut HashMap<u64, SkinnedChannelData>,
) -> HashMap<Entity, Vec<SkinnedChannelData>> {
    let mut joint_channels: HashMap<Entity, Vec<SkinnedChannelData>> = HashMap::new();
    let Some(clip) = player.clips.get(layer.clip_index) else {
        return joint_channels;
    };
    for (channel_index, channel) in clip.channels.iter().enumerate() {
        if !layer.mask.is_empty() {
            let in_mask = channel
                .target_bone_name
                .as_ref()
                .is_some_and(|name| layer.mask.iter().any(|masked| masked == name));
            if !in_mask {
                continue;
            }
        }
        let key = channel_key(player_entity, clip, layer.clip_index, channel_index);
        if let Some(owned) = build_channel(channel, key, clip.source != 0, cache)
            && let Some(target) = player.resolve_target_entity(channel)
        {
            joint_channels.entry(target).or_default().push(owned);
        }
    }
    joint_channels
}

fn resolve_multi_ik_chains(
    player: &AnimationPlayer,
    local_index_of: &HashMap<Entity, usize>,
) -> Vec<nightshade_render::data::config::SkinnedMultiIkChainData> {
    player
        .ik_chains_multi
        .iter()
        .enumerate()
        .filter_map(|(chain_index, chain)| {
            if chain.bones.len() < 2 {
                return None;
            }
            let mut joint_locals = Vec::with_capacity(chain.bones.len());
            for bone in &chain.bones {
                let joint = player.bone_name_to_entity.get(bone)?;
                joint_locals.push(local_index_of.get(joint).copied()? as u32);
            }
            Some(nightshade_render::data::config::SkinnedMultiIkChainData {
                chain_index: chain_index as u32,
                joint_locals,
            })
        })
        .collect()
}

fn resolve_spring_chains(
    world: &nightshade::ecs::Storage,
    player: &AnimationPlayer,
    local_index_of: &HashMap<Entity, usize>,
) -> Vec<nightshade_render::data::config::SkinnedSpringChainData> {
    player
        .spring_chains
        .iter()
        .enumerate()
        .filter_map(|(chain_index, chain)| {
            if chain.bones.len() < 2 {
                return None;
            }
            let mut joint_locals = Vec::with_capacity(chain.bones.len());
            let mut rest_axes = Vec::with_capacity(chain.bones.len());
            let mut bone_lengths = Vec::with_capacity(chain.bones.len());
            for bone in &chain.bones {
                let joint = *player.bone_name_to_entity.get(bone)?;
                joint_locals.push(local_index_of.get(&joint).copied()? as u32);
                let local = world
                    .get::<nightshade_scene::contract::LocalTransform>(joint)
                    .map(|transform| transform.translation)
                    .unwrap_or_else(nalgebra_glm::Vec3::zeros);
                let length = local.magnitude();
                let axis = if length > 1.0e-6 {
                    local / length
                } else {
                    nalgebra_glm::Vec3::new(0.0, 1.0, 0.0)
                };
                rest_axes.push([axis.x, axis.y, axis.z]);
                bone_lengths.push(length);
            }
            Some(nightshade_render::data::config::SkinnedSpringChainData {
                chain_index: chain_index as u32,
                joint_locals,
                rest_axes,
                bone_lengths,
            })
        })
        .collect()
}

fn resolve_aim_constraints(
    player: &AnimationPlayer,
    local_index_of: &HashMap<Entity, usize>,
) -> Vec<nightshade_render::data::config::SkinnedAimConstraintData> {
    player
        .aim_constraints
        .iter()
        .enumerate()
        .filter_map(|(chain_index, constraint)| {
            if constraint.bones.is_empty() {
                return None;
            }
            let mut joint_locals = Vec::with_capacity(constraint.bones.len());
            for bone in &constraint.bones {
                let joint = player.bone_name_to_entity.get(bone)?;
                joint_locals.push(local_index_of.get(joint).copied()? as u32);
            }
            Some(nightshade_render::data::config::SkinnedAimConstraintData {
                chain_index: chain_index as u32,
                joint_locals,
                forward: [
                    constraint.forward.x,
                    constraint.forward.y,
                    constraint.forward.z,
                ],
            })
        })
        .collect()
}

fn build_reference_joint_channels(
    player: &AnimationPlayer,
    player_entity: Entity,
    layer: &AnimationLayer,
    cache: &mut HashMap<u64, SkinnedChannelData>,
) -> HashMap<Entity, Vec<SkinnedChannelData>> {
    let mut joint_channels: HashMap<Entity, Vec<SkinnedChannelData>> = HashMap::new();
    if layer.blend_mode != crate::contract::LayerBlendMode::Additive {
        return joint_channels;
    }
    let Some(reference_index) = layer.reference_clip else {
        return joint_channels;
    };
    let Some(clip) = player.clips.get(reference_index) else {
        return joint_channels;
    };
    for (channel_index, channel) in clip.channels.iter().enumerate() {
        if !layer.mask.is_empty() {
            let in_mask = channel
                .target_bone_name
                .as_ref()
                .is_some_and(|name| layer.mask.iter().any(|masked| masked == name));
            if !in_mask {
                continue;
            }
        }
        let key = channel_key(player_entity, clip, reference_index, channel_index);
        if let Some(owned) = build_channel(channel, key, clip.source != 0, cache)
            && let Some(target) = player.resolve_target_entity(channel)
        {
            joint_channels.entry(target).or_default().push(owned);
        }
    }
    joint_channels
}

fn joint_depth(
    world: &nightshade::ecs::Storage,
    joints: &[Entity],
    local_index_of: &HashMap<Entity, usize>,
    local: usize,
) -> u32 {
    let mut depth = 0u32;
    let mut current = joints[local];
    while let Some(parent) = world
        .get::<nightshade::ecs::ChildOf>(current)
        .map(|child_of| child_of.0)
    {
        if !local_index_of.contains_key(&parent) {
            break;
        }
        depth += 1;
        current = parent;
        if depth > joints.len() as u32 {
            break;
        }
    }
    depth
}

fn signatures(
    world: &nightshade::ecs::Storage,
    skin_entities: &[Entity],
    joint_to_player: &HashMap<Entity, Entity>,
) -> (u64, u64) {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    let mut clips = std::collections::hash_map::DefaultHasher::new();
    for &skin_entity in skin_entities {
        let Some(skin) = world.get::<nightshade_render::components::skin::Skin>(skin_entity) else {
            continue;
        };
        let Some(&player_entity) = skin
            .joints
            .iter()
            .find_map(|joint| joint_to_player.get(joint))
        else {
            continue;
        };
        skin_entity.id.hash(&mut hasher);
        skin_entity.id.hash(&mut clips);
        for joint in &skin.joints {
            joint.id.hash(&mut hasher);
        }
        if let Some(player) = world.get::<crate::contract::AnimationPlayer>(player_entity) {
            player.active_clips.len().hash(&mut clips);
            for active in &player.active_clips {
                active.clip_index.hash(&mut clips);
            }
            for layer in &player.layers {
                layer.clip_index.hash(&mut clips);
                layer.reference_clip.hash(&mut clips);
            }
            player.layers.len().hash(&mut hasher);
            for layer in &player.layers {
                for bone in &layer.mask {
                    bone.hash(&mut hasher);
                }
                matches!(layer.blend_mode, crate::contract::LayerBlendMode::Additive)
                    .hash(&mut hasher);
                0xffu8.hash(&mut hasher);
            }
            player.root_motion.hash(&mut hasher);
            if let Some(bone) = &player.root_motion_bone {
                bone.hash(&mut hasher);
            }
            player.ik_chains.len().hash(&mut hasher);
            for chain in &player.ik_chains {
                chain.root_bone.hash(&mut hasher);
                chain.mid_bone.hash(&mut hasher);
                chain.tip_bone.hash(&mut hasher);
                chain.pole.is_some().hash(&mut hasher);
            }
            player.ik_chains_multi.len().hash(&mut hasher);
            for chain in &player.ik_chains_multi {
                for bone in &chain.bones {
                    bone.hash(&mut hasher);
                }
                chain.iterations.hash(&mut hasher);
                0xfeu8.hash(&mut hasher);
            }
            player.aim_constraints.len().hash(&mut hasher);
            for constraint in &player.aim_constraints {
                for bone in &constraint.bones {
                    bone.hash(&mut hasher);
                }
                0xfdu8.hash(&mut hasher);
            }
            player.spring_chains.len().hash(&mut hasher);
            for chain in &player.spring_chains {
                for bone in &chain.bones {
                    bone.hash(&mut hasher);
                }
                0xfcu8.hash(&mut hasher);
            }
        }
    }
    (hasher.finish(), clips.finish())
}

fn refresh_clip_channels(
    world: &nightshade::ecs::Storage,
    skeletons: &mut [SkinnedSkeletonData],
    cache: &mut HashMap<u64, SkinnedChannelData>,
) {
    for skeleton in skeletons.iter_mut() {
        let Some(skin) =
            world.get::<nightshade_render::components::skin::Skin>(skeleton.skin_entity)
        else {
            continue;
        };
        let player_entity = skeleton.player_entity;
        let player = world.get::<crate::contract::AnimationPlayer>(player_entity);
        let pose_joint_channels = player
            .map(|player| build_active_clip_channels(player, player_entity, cache))
            .unwrap_or_default();
        skeleton.active_count = pose_joint_channels.len() as u32;
        let layer_joint_channels: Vec<HashMap<Entity, Vec<SkinnedChannelData>>> = player
            .map(|player| {
                player
                    .layers
                    .iter()
                    .map(|layer| build_layer_joint_channels(player, player_entity, layer, cache))
                    .collect()
            })
            .unwrap_or_default();
        let reference_joint_channels: Vec<HashMap<Entity, Vec<SkinnedChannelData>>> = player
            .map(|player| {
                player
                    .layers
                    .iter()
                    .map(|layer| {
                        build_reference_joint_channels(player, player_entity, layer, cache)
                    })
                    .collect()
            })
            .unwrap_or_default();
        for joint_data in &mut skeleton.joints_ordered {
            let Some(&joint) = skin.joints.get(joint_data.local_index as usize) else {
                continue;
            };
            joint_data.pose_channels = pose_joint_channels
                .iter()
                .map(|channels| channels.get(&joint).cloned().unwrap_or_default())
                .collect();
            joint_data.layer_channels = layer_joint_channels
                .iter()
                .map(|channels| channels.get(&joint).cloned().unwrap_or_default())
                .collect();
            joint_data.reference_channels = reference_joint_channels
                .iter()
                .map(|channels| channels.get(&joint).cloned().unwrap_or_default())
                .collect();
        }
    }
}

fn skin_armature_parent(world: &nightshade::ecs::Storage, skin_entity: Entity) -> Option<Entity> {
    let skin = world.get::<nightshade_render::components::skin::Skin>(skin_entity)?;
    let local_index_of: HashMap<Entity, usize> = skin
        .joints
        .iter()
        .enumerate()
        .map(|(local, joint)| (*joint, local))
        .collect();
    skin.joints.iter().find_map(|joint| {
        let parent = world
            .get::<nightshade::ecs::ChildOf>(*joint)
            .map(|child_of| child_of.0)?;
        if local_index_of.contains_key(&parent) {
            return None;
        }
        world
            .get::<nightshade_scene::contract::GlobalTransform>(parent)
            .map(|_global| parent)
    })
}
