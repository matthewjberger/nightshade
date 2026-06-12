use crate::ecs::{PoseKeyframe, SandboxWorld};
use nightshade::ecs::animation::components::{
    AnimationClip, AnimationProperty, AnimationSamplerOutput,
};
use nightshade::ecs::transform::queries::query_children;
use nightshade::prelude::*;
use std::collections::HashMap;

const POSE_BLEND_SHARPNESS: f32 = 12.0;
const REACH_ARM_DEGREES: f32 = 90.0;
const IDLE_ARM_DROP_DEGREES: f32 = 65.0;
const IDLE_ELBOW_BEND_DEGREES: f32 = 20.0;

pub fn initialize(sandbox_world: &mut SandboxWorld, world: &mut World, model: Entity) {
    let Some(player) = world.core.get_animation_player(model) else {
        return;
    };
    let bones = player.bone_name_to_entity.clone();
    let node_index_to_entity = player.node_index_to_entity.clone();
    let clips = player.clips.clone();

    let mut rest = Vec::new();
    for &entity in bones.values() {
        if let Some(transform) = world.core.get_local_transform(entity) {
            rest.push((entity, transform.rotation));
        }
    }
    rest.sort_by_key(|(entity, _)| entity.id);

    let mut keyframes = vec![PoseKeyframe {
        name: "rest".to_string(),
        joints: rest.clone(),
    }];

    for clip in &clips {
        let joints = keyframe_from_clip(clip, &bones, &node_index_to_entity);
        if joints.is_empty() {
            continue;
        }
        let name = unique_name(&sanitize_name(&clip.name), &keyframes);
        keyframes.push(PoseKeyframe { name, joints });
    }

    let idle_joints = build_model_space_keyframe(
        world,
        model,
        &rest,
        &bones,
        &[
            ("LeftArm", Vec3::z(), -IDLE_ARM_DROP_DEGREES),
            ("RightArm", Vec3::z(), IDLE_ARM_DROP_DEGREES),
            ("LeftForeArm", Vec3::x(), -IDLE_ELBOW_BEND_DEGREES),
            ("RightForeArm", Vec3::x(), -IDLE_ELBOW_BEND_DEGREES),
        ],
    );
    keyframes.push(PoseKeyframe {
        name: "idle".to_string(),
        joints: idle_joints,
    });

    let reach_joints = build_model_space_keyframe(
        world,
        model,
        &rest,
        &bones,
        &[
            ("LeftArm", Vec3::y(), -REACH_ARM_DEGREES),
            ("RightArm", Vec3::y(), REACH_ARM_DEGREES),
        ],
    );
    keyframes.push(PoseKeyframe {
        name: "reach".to_string(),
        joints: reach_joints,
    });

    sandbox_world.resources.pose.keyframes = keyframes;
}

pub fn set_pose(sandbox_world: &mut SandboxWorld, world: &mut World, name: &str) -> bool {
    let model = sandbox_world.resources.character.model;
    if name == "clip" {
        sandbox_world.resources.pose.active = None;
        if let Some(model) = model
            && let Some(player) = world.core.get_animation_player_mut(model)
            && !player.clips.is_empty()
        {
            player.playing = true;
        }
        return true;
    }
    let Some(index) = sandbox_world
        .resources
        .pose
        .keyframes
        .iter()
        .position(|keyframe| keyframe.name == name)
    else {
        return false;
    };
    sandbox_world.resources.pose.active = Some(index);
    if let Some(model) = model
        && let Some(player) = world.core.get_animation_player_mut(model)
    {
        player.playing = false;
    }
    true
}

pub fn update(sandbox_world: &SandboxWorld, world: &mut World) {
    let pose = &sandbox_world.resources.pose;
    let Some(active) = pose.active else {
        return;
    };
    let Some(keyframe) = pose.keyframes.get(active) else {
        return;
    };
    let delta_time = world.resources.window.timing.delta_time;
    let blend = 1.0 - (-POSE_BLEND_SHARPNESS * delta_time).exp();
    for (entity, target) in &keyframe.joints {
        let Some(transform) = world.core.get_local_transform_mut(*entity) else {
            continue;
        };
        let mut target = *target;
        if nalgebra_glm::quat_dot(&transform.rotation, &target) < 0.0 {
            target = -target;
        }
        transform.rotation =
            nalgebra_glm::quat_normalize(&(transform.rotation * (1.0 - blend) + target * blend));
        mark_local_transform_dirty(world, *entity);
    }
}

fn keyframe_from_clip(
    clip: &AnimationClip,
    bones: &HashMap<String, Entity>,
    node_index_to_entity: &HashMap<usize, Entity>,
) -> Vec<(Entity, nalgebra_glm::Quat)> {
    let mut joints = Vec::new();
    for channel in &clip.channels {
        if !matches!(channel.target_property, AnimationProperty::Rotation) {
            continue;
        }
        let entity = channel
            .target_bone_name
            .as_ref()
            .and_then(|name| bones.get(name))
            .or_else(|| node_index_to_entity.get(&channel.target_node));
        let Some(&entity) = entity else {
            continue;
        };
        let rotation = match &channel.sampler.output {
            AnimationSamplerOutput::Quat(values) => values.first().copied(),
            AnimationSamplerOutput::CubicSplineQuat { values, .. } => values.first().copied(),
            _ => None,
        };
        if let Some(rotation) = rotation {
            joints.push((entity, rotation));
        }
    }
    joints
}

/// Build a keyframe by applying model-space rotation deltas to named
/// joints. Children follow re-posed parents, so a delta on a forearm
/// composes correctly with a delta on the arm above it.
fn build_model_space_keyframe(
    world: &World,
    model: Entity,
    rest: &[(Entity, nalgebra_glm::Quat)],
    bones: &HashMap<String, Entity>,
    deltas: &[(&str, Vec3, f32)],
) -> Vec<(Entity, nalgebra_glm::Quat)> {
    let mut delta_by_entity: HashMap<Entity, nalgebra_glm::Quat> = HashMap::new();
    for (bone_suffix, axis, degrees) in deltas {
        let Some(entity) = find_bone(bones, bone_suffix) else {
            continue;
        };
        let rotation = nalgebra_glm::quat_angle_axis(degrees.to_radians(), axis);
        delta_by_entity
            .entry(entity)
            .and_modify(|existing| *existing = rotation * *existing)
            .or_insert(rotation);
    }

    let rest_by_entity: HashMap<Entity, nalgebra_glm::Quat> = rest.iter().copied().collect();
    let mut joints = Vec::new();
    let mut stack = vec![(model, nalgebra_glm::quat_identity())];
    while let Some((entity, posed_parent)) = stack.pop() {
        let rest_local = rest_by_entity
            .get(&entity)
            .copied()
            .or_else(|| {
                world
                    .core
                    .get_local_transform(entity)
                    .map(|transform| transform.rotation)
            })
            .unwrap_or_else(nalgebra_glm::quat_identity);
        let local = if let Some(delta) = delta_by_entity.get(&entity) {
            nalgebra_glm::quat_inverse(&posed_parent) * delta * posed_parent * rest_local
        } else {
            rest_local
        };
        if rest_by_entity.contains_key(&entity) {
            joints.push((entity, local));
        }
        let posed_rotation = posed_parent * local;
        for child in query_children(world, entity) {
            stack.push((child, posed_rotation));
        }
    }
    joints
}

fn find_bone(bones: &HashMap<String, Entity>, suffix: &str) -> Option<Entity> {
    bones
        .iter()
        .find(|(name, _)| {
            name.as_str() == suffix
                || name
                    .rsplit_once(':')
                    .is_some_and(|(_, short_name)| short_name == suffix)
        })
        .map(|(_, &entity)| entity)
}

fn sanitize_name(clip_name: &str) -> String {
    let sanitized: String = clip_name
        .to_lowercase()
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character
            } else {
                '_'
            }
        })
        .collect();
    if sanitized.is_empty() {
        "clip_pose".to_string()
    } else {
        sanitized
    }
}

fn unique_name(base: &str, keyframes: &[PoseKeyframe]) -> String {
    if !keyframes.iter().any(|keyframe| keyframe.name == base) {
        return base.to_string();
    }
    let mut counter = 2;
    loop {
        let candidate = format!("{base}_{counter}");
        if !keyframes.iter().any(|keyframe| keyframe.name == candidate) {
            return candidate;
        }
        counter += 1;
    }
}
