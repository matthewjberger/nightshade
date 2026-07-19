use crate::ecs::animation::components::{
    AnimationChannel, AnimationInterpolation, AnimationLayer, AnimationPlayer, AnimationProperty,
    AnimationSamplerOutput,
};
use crate::ecs::world::CORE;
use crate::ecs::world::{ANIMATION_PLAYER, Entity, SKIN, World};
use crate::render::config::{SkinnedChannelData, SkinnedJointData, SkinnedSkeletonData};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

const PROPERTY_TRANSLATION: u32 = 0;
const PROPERTY_ROTATION: u32 = 1;
const PROPERTY_SCALE: u32 = 2;

/// Resolves the scene's skinned-animation state into
/// `renderer_state.render_animation` so the skinned-mesh compute driver builds
/// its GPU buffers without reading ECS animation, skin, parent, or transform
/// components. The static signature is computed only on frames where a cheap
/// change gate fires (an `AnimationPlayer` or `Skin` change tick, a
/// skinned-set generation bump, or a played-clip change, which catches
/// raw-write clip finishes the ticks cannot see), and the heavy per-skeleton
/// payload rebuilds only when that signature actually moved. The runtime maps
/// refresh every frame into reused allocations.
pub fn render_sync_animation_system(world: &mut World) {
    let since = world
        .res::<crate::ecs::graphics::scene_sync::RenderSceneSync>()
        .animation_cursor;
    let players_ticked = world.ecs.worlds[CORE]
        .query_entities_changed_since(ANIMATION_PLAYER, since)
        .next()
        .is_some();
    let skins_ticked = world.ecs.worlds[CORE]
        .query_entities_changed_since(SKIN, since)
        .next()
        .is_some();
    world
        .res_mut::<crate::ecs::graphics::scene_sync::RenderSceneSync>()
        .animation_cursor = world
        .res_mut::<crate::ecs::graphics::scene_sync::RenderSceneSync>()
        .data_cursor;

    let mut player_runtime = std::mem::take(
        &mut world
            .res_mut::<crate::render::config::RendererState>()
            .render_animation
            .player_runtime,
    );
    player_runtime.clear();
    let mut player_clips = std::mem::take(
        &mut world
            .res_mut::<crate::ecs::graphics::scene_sync::RenderSceneSync>()
            .animation_player_clips_scratch,
    );
    player_clips.clear();

    for (player_entity, player) in world.query_ref::<&AnimationPlayer>().iter() {
        if !player.playing || player.play_all || player.get_current_clip().is_none() {
            continue;
        }
        player_runtime.insert(
            player_entity,
            [player.time, player.blend_from_time, player.blend_factor],
        );
        player_clips.insert(
            player_entity,
            (
                player.current_clip.map(|index| index as i64).unwrap_or(-1),
                player
                    .blend_from_clip
                    .map(|index| index as i64)
                    .unwrap_or(-1),
            ),
        );
    }

    let skinned_generation = world
        .res::<crate::render::config::RendererState>()
        .skinned_generation;
    let gate_fired = {
        let sync = &mut world.res_mut::<crate::ecs::graphics::scene_sync::RenderSceneSync>();
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
        let joint_to_player = collect_joint_to_player(world);
        let mut skin_entities: Vec<Entity> = world.ecs.worlds[CORE].query_entities(SKIN).collect();
        skin_entities.sort_by_key(|entity| (entity.id, entity.generation));
        let signature = static_signature(world, &skin_entities, &joint_to_player, &player_clips);
        let snapshot = &mut world
            .res_mut::<crate::render::config::RendererState>()
            .render_animation;
        if signature != snapshot.signature {
            snapshot.signature = signature;
            build_skeletons(world, &skin_entities, &joint_to_player)
        } else {
            std::mem::take(
                &mut world
                    .res_mut::<crate::render::config::RendererState>()
                    .render_animation
                    .skeletons,
            )
        }
    } else {
        std::mem::take(
            &mut world
                .res_mut::<crate::render::config::RendererState>()
                .render_animation
                .skeletons,
        )
    };

    {
        let sync = &mut world.res_mut::<crate::ecs::graphics::scene_sync::RenderSceneSync>();
        sync.animation_player_clips_scratch =
            std::mem::replace(&mut sync.animation_player_clips, player_clips);
    }

    let mut layer_runtime = std::mem::take(
        &mut world
            .res_mut::<crate::render::config::RendererState>()
            .render_animation
            .layer_runtime,
    );
    layer_runtime.clear();
    let mut armature_roots = std::mem::take(
        &mut world
            .res_mut::<crate::render::config::RendererState>()
            .render_animation
            .armature_roots,
    );
    armature_roots.clear();
    for skeleton in &skeletons {
        let root = skeleton
            .armature_parent
            .and_then(|parent| {
                world.get::<crate::ecs::transform::components::GlobalTransform>(parent)
            })
            .map(|global| global.0)
            .unwrap_or_else(nalgebra_glm::Mat4::identity);
        armature_roots.insert(skeleton.skin_entity, root);
        if let Some(player) =
            world.get::<crate::ecs::animation::components::AnimationPlayer>(skeleton.player_entity)
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

    let snapshot = &mut world
        .res_mut::<crate::render::config::RendererState>()
        .render_animation;
    snapshot.skeletons = skeletons;
    snapshot.player_runtime = player_runtime;
    snapshot.layer_runtime = layer_runtime;
    snapshot.armature_roots = armature_roots;
}

fn collect_joint_to_player(world: &World) -> HashMap<Entity, Entity> {
    let mut joint_to_player: HashMap<Entity, Entity> = HashMap::new();
    for (player_entity, player) in world.query_ref::<&AnimationPlayer>().iter() {
        if !player.playing || player.play_all {
            continue;
        }
        let Some(current) = player.get_current_clip() else {
            continue;
        };
        for channel in &current.channels {
            if gpu_property(channel.target_property).is_some()
                && let Some(target) = player.resolve_target_entity(channel)
            {
                joint_to_player.entry(target).or_insert(player_entity);
            }
        }
    }
    joint_to_player
}

fn build_skeletons(
    world: &World,
    skin_entities: &[Entity],
    joint_to_player: &HashMap<Entity, Entity>,
) -> Vec<SkinnedSkeletonData> {
    let mut joint_cur: HashMap<Entity, Vec<SkinnedChannelData>> = HashMap::new();
    let mut joint_blend: HashMap<Entity, Vec<SkinnedChannelData>> = HashMap::new();
    for (_, player) in world.query_ref::<&AnimationPlayer>().iter() {
        if !player.playing || player.play_all {
            continue;
        }
        if let Some(current) = player.get_current_clip() {
            for channel in &current.channels {
                if let Some(owned) = build_channel(channel)
                    && let Some(target) = player.resolve_target_entity(channel)
                {
                    joint_cur.entry(target).or_default().push(owned);
                }
            }
        }
        if let Some(from_clip) = player
            .blend_from_clip
            .and_then(|index| player.clips.get(index))
        {
            for channel in &from_clip.channels {
                if let Some(owned) = build_channel(channel)
                    && let Some(target) = player.resolve_target_entity(channel)
                {
                    joint_blend.entry(target).or_default().push(owned);
                }
            }
        }
    }

    let mut skeletons: Vec<SkinnedSkeletonData> = Vec::new();
    for &skin_entity in skin_entities {
        let Some(skin) = world.get::<crate::ecs::skin::components::Skin>(skin_entity) else {
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

        let layer_joint_channels: Vec<HashMap<Entity, Vec<SkinnedChannelData>>> = world.ecs.worlds
            [CORE]
            .get::<crate::ecs::animation::components::AnimationPlayer>(player_entity)
            .map(|player| {
                player
                    .layers
                    .iter()
                    .map(|layer| build_layer_joint_channels(player, layer))
                    .collect()
            })
            .unwrap_or_default();
        let layer_count = layer_joint_channels.len() as u32;

        let mut joints_ordered: Vec<SkinnedJointData> = Vec::with_capacity(order.len());
        for &local in &order {
            let joint = skin.joints[local];
            let rest = world
                .get::<crate::ecs::transform::components::LocalTransform>(joint)
                .copied()
                .unwrap_or_default();
            let parent_local = world
                .get::<nightshade_ecs::dynamic::ChildOf>(joint)
                .map(|child_of| child_of.0)
                .and_then(|parent| local_index_of.get(&parent).copied());
            let cur_channels = joint_cur.get(&joint).cloned().unwrap_or_default();
            let blend_channels = joint_blend.get(&joint).cloned().unwrap_or_default();
            let layer_channels: Vec<Vec<SkinnedChannelData>> = layer_joint_channels
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
                cur_channels,
                blend_channels,
                layer_channels,
            });
        }

        skeletons.push(SkinnedSkeletonData {
            skin_entity,
            player_entity,
            armature_parent: skin_armature_parent(world, skin_entity),
            joint_count: skin.joints.len() as u32,
            layer_count,
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

fn build_channel(channel: &AnimationChannel) -> Option<SkinnedChannelData> {
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
    Some(SkinnedChannelData {
        property,
        interpolation,
        input: sampler.input.clone(),
        values,
        stride,
    })
}

fn build_layer_joint_channels(
    player: &AnimationPlayer,
    layer: &AnimationLayer,
) -> HashMap<Entity, Vec<SkinnedChannelData>> {
    let mut joint_channels: HashMap<Entity, Vec<SkinnedChannelData>> = HashMap::new();
    let Some(clip) = player.clips.get(layer.clip_index) else {
        return joint_channels;
    };
    for channel in &clip.channels {
        if !layer.mask.is_empty() {
            let in_mask = channel
                .target_bone_name
                .as_ref()
                .is_some_and(|name| layer.mask.iter().any(|masked| masked == name));
            if !in_mask {
                continue;
            }
        }
        if let Some(owned) = build_channel(channel)
            && let Some(target) = player.resolve_target_entity(channel)
        {
            joint_channels.entry(target).or_default().push(owned);
        }
    }
    joint_channels
}

fn joint_depth(
    world: &World,
    joints: &[Entity],
    local_index_of: &HashMap<Entity, usize>,
    local: usize,
) -> u32 {
    let mut depth = 0u32;
    let mut current = joints[local];
    while let Some(parent) = world
        .get::<nightshade_ecs::dynamic::ChildOf>(current)
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

fn static_signature(
    world: &World,
    skin_entities: &[Entity],
    joint_to_player: &HashMap<Entity, Entity>,
    player_clips: &HashMap<Entity, (i64, i64)>,
) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    for &skin_entity in skin_entities {
        let Some(skin) = world.get::<crate::ecs::skin::components::Skin>(skin_entity) else {
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
        for joint in &skin.joints {
            joint.id.hash(&mut hasher);
        }
        if let Some(clips) = player_clips.get(&player_entity) {
            clips.hash(&mut hasher);
        }
        if let Some(player) =
            world.get::<crate::ecs::animation::components::AnimationPlayer>(player_entity)
        {
            player.layers.len().hash(&mut hasher);
            for layer in &player.layers {
                layer.clip_index.hash(&mut hasher);
                for bone in &layer.mask {
                    bone.hash(&mut hasher);
                }
                0xffu8.hash(&mut hasher);
            }
        }
    }
    hasher.finish()
}

fn skin_armature_parent(world: &World, skin_entity: Entity) -> Option<Entity> {
    let skin = world.get::<crate::ecs::skin::components::Skin>(skin_entity)?;
    let local_index_of: HashMap<Entity, usize> = skin
        .joints
        .iter()
        .enumerate()
        .map(|(local, joint)| (*joint, local))
        .collect();
    skin.joints.iter().find_map(|joint| {
        let parent = world
            .get::<nightshade_ecs::dynamic::ChildOf>(*joint)
            .map(|child_of| child_of.0)?;
        if local_index_of.contains_key(&parent) {
            return None;
        }
        world
            .get::<crate::ecs::transform::components::GlobalTransform>(parent)
            .map(|_global| parent)
    })
}
