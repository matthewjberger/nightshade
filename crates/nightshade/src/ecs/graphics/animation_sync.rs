use crate::ecs::animation::components::{
    AnimationChannel, AnimationInterpolation, AnimationLayer, AnimationPlayer, AnimationProperty,
    AnimationSamplerOutput,
};
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
/// components. The heavy per-skeleton payload is rebuilt only when the static
/// signature changes; the runtime maps refresh every frame.
pub fn render_sync_animation_system(world: &mut World) {
    let mut joint_to_player: HashMap<Entity, Entity> = HashMap::new();
    let mut player_runtime: HashMap<Entity, [f32; 3]> = HashMap::new();
    let mut player_clips: HashMap<Entity, (i64, i64)> = HashMap::new();

    world
        .core
        .query()
        .with(ANIMATION_PLAYER)
        .iter(|player_entity, table, idx| {
            let player = &table.animation_player[idx];
            if !player.playing || player.play_all {
                return;
            }
            let Some(current) = player.get_current_clip() else {
                return;
            };
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
            for channel in &current.channels {
                if gpu_property(channel.target_property).is_some()
                    && let Some(target) = player.resolve_target_entity(channel)
                {
                    joint_to_player.entry(target).or_insert(player_entity);
                }
            }
        });

    let mut skin_entities: Vec<Entity> = world.core.query_entities(SKIN).collect();
    skin_entities.sort_by_key(|entity| (entity.id, entity.generation));

    let signature = static_signature(world, &skin_entities, &joint_to_player, &player_clips);
    let previous_signature = world.resources.renderer_state.render_animation.signature;

    let skeletons = if signature != previous_signature {
        build_skeletons(world, &skin_entities, &joint_to_player)
    } else {
        std::mem::take(&mut world.resources.renderer_state.render_animation.skeletons)
    };

    let mut layer_runtime: HashMap<(Entity, usize), [f32; 2]> = HashMap::new();
    let mut armature_roots: HashMap<Entity, nalgebra_glm::Mat4> = HashMap::new();
    for skeleton in &skeletons {
        armature_roots.insert(
            skeleton.skin_entity,
            skin_armature_root(world, skeleton.skin_entity),
        );
        if let Some(player) = world.core.get_animation_player(skeleton.player_entity) {
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

    let snapshot = &mut world.resources.renderer_state.render_animation;
    snapshot.signature = signature;
    snapshot.skeletons = skeletons;
    snapshot.player_runtime = player_runtime;
    snapshot.layer_runtime = layer_runtime;
    snapshot.armature_roots = armature_roots;
}

fn build_skeletons(
    world: &World,
    skin_entities: &[Entity],
    joint_to_player: &HashMap<Entity, Entity>,
) -> Vec<SkinnedSkeletonData> {
    let mut joint_cur: HashMap<Entity, Vec<SkinnedChannelData>> = HashMap::new();
    let mut joint_blend: HashMap<Entity, Vec<SkinnedChannelData>> = HashMap::new();
    world
        .core
        .query()
        .with(ANIMATION_PLAYER)
        .iter(|_, table, idx| {
            let player = &table.animation_player[idx];
            if !player.playing || player.play_all {
                return;
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
        });

    let mut skeletons: Vec<SkinnedSkeletonData> = Vec::new();
    for &skin_entity in skin_entities {
        let Some(skin) = world.core.get_skin(skin_entity) else {
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

        let layer_joint_channels: Vec<HashMap<Entity, Vec<SkinnedChannelData>>> = world
            .core
            .get_animation_player(player_entity)
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
                .core
                .get_local_transform(joint)
                .copied()
                .unwrap_or_default();
            let parent_local = world
                .core
                .get_parent(joint)
                .and_then(|parent| parent.0)
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
    while let Some(parent) = world.core.get_parent(current).and_then(|parent| parent.0) {
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

fn skin_armature_root(world: &World, skin_entity: Entity) -> nalgebra_glm::Mat4 {
    let Some(skin) = world.core.get_skin(skin_entity) else {
        return nalgebra_glm::Mat4::identity();
    };
    let local_index_of: HashMap<Entity, usize> = skin
        .joints
        .iter()
        .enumerate()
        .map(|(local, joint)| (*joint, local))
        .collect();
    skin.joints
        .iter()
        .find_map(|joint| {
            let parent = world.core.get_parent(*joint).and_then(|parent| parent.0)?;
            if local_index_of.contains_key(&parent) {
                return None;
            }
            world
                .core
                .get_global_transform(parent)
                .map(|global| global.0)
        })
        .unwrap_or_else(nalgebra_glm::Mat4::identity)
}

fn static_signature(
    world: &World,
    skin_entities: &[Entity],
    joint_to_player: &HashMap<Entity, Entity>,
    player_clips: &HashMap<Entity, (i64, i64)>,
) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    for &skin_entity in skin_entities {
        let Some(skin) = world.core.get_skin(skin_entity) else {
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
        if let Some(player) = world.core.get_animation_player(player_entity) {
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
