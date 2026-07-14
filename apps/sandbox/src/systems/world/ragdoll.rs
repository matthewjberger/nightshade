use crate::ecs::{RagdollLink, RagdollMode, SandboxResources};
use crate::systems::world::ik::{rotation_between, world_pose_of};
use crate::systems::world::pose::{find_bone, joint_rotation, quat_to_rotation_vector};
use nightshade::ecs::transform::queries::query_children;
use nightshade::plugins::physics::joints::{SphericalJoint, create_spherical_joint};
use nightshade::prelude::*;
use std::collections::HashMap;

const SEGMENTS: &[(&str, Option<&str>, f32, f32)] = &[
    ("Hips", Some("Spine1"), 0.13, 12.0),
    ("Spine1", Some("Neck"), 0.13, 14.0),
    ("Head", None, 0.1, 5.0),
    ("LeftUpLeg", Some("LeftLeg"), 0.09, 7.0),
    ("LeftLeg", Some("LeftFoot"), 0.07, 4.0),
    ("RightUpLeg", Some("RightLeg"), 0.09, 7.0),
    ("RightLeg", Some("RightFoot"), 0.07, 4.0),
    ("LeftArm", Some("LeftForeArm"), 0.06, 3.0),
    ("LeftForeArm", Some("LeftHand"), 0.05, 2.0),
    ("RightArm", Some("RightForeArm"), 0.06, 3.0),
    ("RightForeArm", Some("RightHand"), 0.05, 2.0),
];

const CONNECTIONS: &[(&str, &str)] = &[
    ("Hips", "Spine1"),
    ("Spine1", "Head"),
    ("Spine1", "LeftArm"),
    ("Spine1", "RightArm"),
    ("Hips", "LeftUpLeg"),
    ("Hips", "RightUpLeg"),
    ("LeftUpLeg", "LeftLeg"),
    ("RightUpLeg", "RightLeg"),
    ("LeftArm", "LeftForeArm"),
    ("RightArm", "RightForeArm"),
];

const HEAD_SEGMENT_LENGTH: f32 = 0.22;
const CAPSULE_FILL: f32 = 0.85;
const MATCH_STIFFNESS: f32 = 20.0;
const MATCH_DAMPING: f32 = 2.0;
const FLAIL_FREQUENCY: f32 = 2.2;
const FLAIL_SPEED: f32 = 6.0;
const PROTECT_SPEED: f32 = 2.5;
const FORWARD_PROTECT_SPEED: f32 = 4.0;
const SETTLE_SPEED: f32 = 0.7;
const SETTLE_SECONDS: f32 = 0.9;
const HIPS_FEET_DROP: f32 = 0.55;

pub fn set_mode(sandbox_world: &mut SandboxResources, world: &mut World, mode: RagdollMode) {
    let current = sandbox_world.ragdoll.mode;
    if current == mode {
        return;
    }
    if mode == RagdollMode::Off {
        teardown(sandbox_world, world);
        return;
    }
    if sandbox_world.ragdoll.links.is_empty() {
        build(sandbox_world, world);
    }
    if sandbox_world.ragdoll.links.is_empty() {
        return;
    }
    if mode == RagdollMode::Match {
        capture_match_targets(sandbox_world, world);
    }
    let state = &mut sandbox_world.ragdoll;
    state.mode = mode;
    state.settle_timer = 0.0;
}

pub fn activate_from_impact(sandbox_world: &mut SandboxResources, world: &mut World) {
    set_mode(sandbox_world, world, RagdollMode::Active);
    sandbox_world.ragdoll.auto_recover = true;
}

pub fn update(sandbox_world: &mut SandboxResources, world: &mut World, body: Entity) {
    let delta_time = world.resources.window.timing.delta_time;
    sandbox_world.ragdoll.flail_clock += delta_time;

    match sandbox_world.ragdoll.mode {
        RagdollMode::Off => return,
        RagdollMode::Limp => {}
        RagdollMode::Match => drive_toward_captured_targets(sandbox_world, world),
        RagdollMode::Active => drive_toward_pose(sandbox_world, world, body),
    }

    sync_bones_from_bodies(sandbox_world, world, body);

    let hips_velocity = hips_link(sandbox_world)
        .and_then(|link| physics_world_linear_velocity(&world.resources.physics, link.body))
        .unwrap_or_else(Vec3::zeros);
    let state = &mut sandbox_world.ragdoll;
    if state.auto_recover {
        if hips_velocity.norm() < SETTLE_SPEED {
            state.settle_timer += delta_time;
        } else {
            state.settle_timer = 0.0;
        }
        if state.settle_timer > SETTLE_SECONDS {
            state.auto_recover = false;
            teardown(sandbox_world, world);
        }
    }
}

fn hips_link(sandbox_world: &SandboxResources) -> Option<&RagdollLink> {
    let hips = sandbox_world.pose.hips?;
    sandbox_world
        .ragdoll
        .links
        .iter()
        .find(|link| link.bone == hips)
}

fn bones_of_model(
    sandbox_world: &SandboxResources,
    world: &World,
) -> Option<HashMap<String, Entity>> {
    let model = sandbox_world.character.model?;
    world
        .get::<nightshade::ecs::animation::components::AnimationPlayer>(model)
        .map(|player| player.bone_name_to_entity.clone())
}

fn build(sandbox_world: &mut SandboxResources, world: &mut World) {
    let Some(body) = sandbox_world.character.body else {
        return;
    };
    let Some(bones) = bones_of_model(sandbox_world, world) else {
        return;
    };
    let launch_velocity = sandbox_world.character.velocity;

    let mut links = Vec::new();
    let mut link_by_suffix: HashMap<&str, usize> = HashMap::new();
    for (suffix, end_suffix, radius, mass) in SEGMENTS {
        let Some(bone) = find_bone(&bones, suffix) else {
            continue;
        };
        let Some(start) = world_pose_of(world, body, bone) else {
            continue;
        };
        let end_position = end_suffix
            .and_then(|end| find_bone(&bones, end))
            .and_then(|end_bone| world_pose_of(world, body, end_bone))
            .map(|pose| pose.position)
            .unwrap_or_else(|| {
                start.position
                    + nalgebra_glm::quat_rotate_vec3(&start.rotation, &Vec3::y())
                        * HEAD_SEGMENT_LENGTH
            });
        let segment = end_position - start.position;
        let length = segment.norm();
        if length < 0.05 {
            continue;
        }
        let direction = segment / length;
        let center = start.position + segment * 0.5;
        let body_rotation = rotation_between(Vec3::y(), direction);

        let mut rigid_body = RigidBodyComponent::new_dynamic()
            .with_translation(center.x, center.y, center.z)
            .with_rotation(
                body_rotation.i,
                body_rotation.j,
                body_rotation.k,
                body_rotation.w,
            )
            .with_mass(*mass);
        rigid_body.linvel = [launch_velocity.x, launch_velocity.y, launch_velocity.z];
        let half_height = ((length * CAPSULE_FILL) * 0.5 - radius).max(0.02);
        let proxy = world.spawn_with((
            Name(format!("Ragdoll {suffix}")),
            LocalTransform {
                translation: center,
                rotation: body_rotation,
                ..Default::default()
            },
            GlobalTransform::default(),
            rigid_body,
            ColliderComponent::new_capsule(half_height, *radius)
                .with_friction(0.8)
                .with_restitution(0.05),
            PhysicsInterpolation {
                previous_translation: center,
                current_translation: center,
                previous_rotation: body_rotation,
                current_rotation: body_rotation,
                enabled: true,
            },
        ));

        link_by_suffix.insert(suffix, links.len());
        links.push(RagdollLink {
            body: proxy,
            bone,
            rotation_offset: nalgebra_glm::quat_inverse(&body_rotation) * start.rotation,
            target_world: start.rotation,
            mass: *mass,
        });
    }

    for (parent_suffix, child_suffix) in CONNECTIONS {
        let (Some(&parent_index), Some(&child_index)) = (
            link_by_suffix.get(parent_suffix),
            link_by_suffix.get(child_suffix),
        ) else {
            continue;
        };
        let Some(anchor) =
            world_pose_of(world, body, links[child_index].bone).map(|pose| pose.position)
        else {
            continue;
        };
        let parent_proxy = links[parent_index].body;
        let child_proxy = links[child_index].body;
        let anchor_local = |proxy: Entity, world: &World| -> Vec3 {
            let transform =
                world.get::<nightshade::ecs::transform::components::LocalTransform>(proxy);
            let rotation = transform
                .map(|transform| transform.rotation)
                .unwrap_or_else(nalgebra_glm::quat_identity);
            let position = transform
                .map(|transform| transform.translation)
                .unwrap_or_else(Vec3::zeros);
            nalgebra_glm::quat_rotate_vec3(
                &nalgebra_glm::quat_inverse(&rotation),
                &(anchor - position),
            )
        };
        let local_anchor_parent = anchor_local(parent_proxy, world);
        let local_anchor_child = anchor_local(child_proxy, world);
        create_spherical_joint(
            world,
            parent_proxy,
            child_proxy,
            SphericalJoint::new()
                .with_local_anchor1(local_anchor_parent)
                .with_local_anchor2(local_anchor_child),
        );
    }

    sandbox_world.ragdoll.links = links;
}

fn teardown(sandbox_world: &mut SandboxResources, world: &mut World) {
    let hips_pose = hips_link(sandbox_world).and_then(|link| {
        world
            .get::<nightshade::ecs::transform::components::LocalTransform>(link.body)
            .map(|transform| transform.translation)
            .map(|position| {
                (
                    position,
                    physics_world_linear_velocity(&world.resources.physics, link.body)
                        .unwrap_or_else(Vec3::zeros),
                )
            })
    });

    let bodies: Vec<Entity> = sandbox_world
        .ragdoll
        .links
        .iter()
        .map(|link| link.body)
        .collect();
    if !bodies.is_empty() {
        despawn_entities_with_cache_cleanup(world, &bodies);
    }
    let state = &mut sandbox_world.ragdoll;
    state.links.clear();
    state.mode = RagdollMode::Off;
    state.auto_recover = false;
    state.settle_timer = 0.0;

    if let Some((hips_position, hips_velocity)) = hips_pose {
        let feet = hips_position - Vec3::new(0.0, HIPS_FEET_DROP, 0.0);
        let character = &mut sandbox_world.character;
        character.velocity = hips_velocity;
        character.grounded = false;
        if let Some(body) = character.body
            && let Some(transform) =
                world.get_mut::<nightshade::ecs::transform::components::LocalTransform>(body)
        {
            transform.translation = feet;
        }
    }
}

fn capture_match_targets(sandbox_world: &mut SandboxResources, world: &mut World) {
    let Some(body) = sandbox_world.character.body else {
        return;
    };
    let targets: Vec<(Entity, nalgebra_glm::Quat)> = sandbox_world
        .ragdoll
        .links
        .iter()
        .filter_map(|link| {
            world_pose_of(world, body, link.bone).map(|pose| (link.bone, pose.rotation))
        })
        .collect();
    for link in &mut sandbox_world.ragdoll.links {
        if let Some((_, rotation)) = targets.iter().find(|(bone, _)| *bone == link.bone) {
            link.target_world = *rotation;
        }
    }
}

fn drive_toward_captured_targets(sandbox_world: &SandboxResources, world: &mut World) {
    for link in &sandbox_world.ragdoll.links {
        let target = link.target_world;
        apply_match_torque(world, link, target);
    }
}

/// Active ragdoll: pick a target pose by how the body is moving, then
/// drive every limb toward it with joint torques. Fast flight flails
/// between two poses, falling near the ground curls up, and fast
/// horizontal travel puts the arms out front.
fn drive_toward_pose(sandbox_world: &SandboxResources, world: &mut World, body: Entity) {
    let pose = &sandbox_world.pose;
    let velocity = hips_link(sandbox_world)
        .and_then(|link| physics_world_linear_velocity(&world.resources.physics, link.body))
        .unwrap_or_else(Vec3::zeros);
    let speed = velocity.norm();
    let horizontal_speed = Vec3::new(velocity.x, 0.0, velocity.z).norm();

    let keyframe_index = if speed > FLAIL_SPEED {
        let phase =
            (sandbox_world.ragdoll.flail_clock * FLAIL_FREQUENCY * std::f32::consts::TAU).sin();
        if phase > 0.0 {
            pose.air_index
        } else {
            pose.keyframes
                .iter()
                .position(|keyframe| keyframe.name == "reach")
        }
    } else if horizontal_speed > FORWARD_PROTECT_SPEED {
        pose.keyframes
            .iter()
            .position(|keyframe| keyframe.name == "reach")
    } else if speed > PROTECT_SPEED {
        pose.tuck_forward_index
    } else {
        None
    };
    let Some(keyframe) = keyframe_index.and_then(|index| pose.keyframes.get(index)) else {
        return;
    };

    let Some(model) = sandbox_world.character.model else {
        return;
    };
    let link_bones: Vec<Entity> = sandbox_world
        .ragdoll
        .links
        .iter()
        .map(|link| link.bone)
        .collect();
    let target_worlds = keyframe_world_rotations(world, body, model, &keyframe.joints, &link_bones);
    for link in &sandbox_world.ragdoll.links {
        if let Some(&target) = target_worlds.get(&link.bone) {
            apply_match_torque(world, link, target);
        }
    }
}

fn apply_match_torque(world: &mut World, link: &RagdollLink, target_world: nalgebra_glm::Quat) {
    let Some(current_bone_world) = world
        .get::<nightshade::ecs::transform::components::LocalTransform>(link.body)
        .map(|transform| transform.rotation * link.rotation_offset)
    else {
        return;
    };
    let mut target = target_world;
    if nalgebra_glm::quat_dot(&current_bone_world, &target) < 0.0 {
        target = -target;
    }
    let error =
        quat_to_rotation_vector(&(target * nalgebra_glm::quat_inverse(&current_bone_world)));
    let angular_velocity = physics_world_angular_velocity(&world.resources.physics, link.body)
        .unwrap_or_else(Vec3::zeros);
    let torque =
        error * (MATCH_STIFFNESS * link.mass) - angular_velocity * (MATCH_DAMPING * link.mass);
    physics_world_apply_torque(&mut world.resources.physics, link.body, torque);
}

/// World rotations the mapped bones would have if the given keyframe
/// were applied, walking the skeleton with keyframe locals.
fn keyframe_world_rotations(
    world: &World,
    body: Entity,
    model: Entity,
    keyframe_joints: &[(Entity, nalgebra_glm::Quat)],
    wanted: &[Entity],
) -> HashMap<Entity, nalgebra_glm::Quat> {
    let body_rotation = world
        .get::<nightshade::ecs::transform::components::LocalTransform>(body)
        .map(|transform| transform.rotation)
        .unwrap_or_else(nalgebra_glm::quat_identity);
    let mut results = HashMap::new();
    let mut stack = vec![(model, body_rotation)];
    while let Some((entity, parent_rotation)) = stack.pop() {
        let local = joint_rotation(keyframe_joints, entity).unwrap_or_else(|| {
            world
                .get::<nightshade::ecs::transform::components::LocalTransform>(entity)
                .map(|transform| transform.rotation)
                .unwrap_or_else(nalgebra_glm::quat_identity)
        });
        let rotation = parent_rotation * local;
        if wanted.contains(&entity) {
            results.insert(entity, rotation);
        }
        for child in query_children(world, entity) {
            stack.push((child, rotation));
        }
    }
    results
}

/// Copy the physics bodies back onto the skeleton: the character body
/// follows the pelvis, the hips bone translates to the pelvis body,
/// and every mapped bone takes its orientation from its proxy.
fn sync_bones_from_bodies(sandbox_world: &SandboxResources, world: &mut World, body: Entity) {
    let Some(model) = sandbox_world.character.model else {
        return;
    };
    let hips_bone = sandbox_world.pose.hips;
    let hips_position = hips_link(sandbox_world).and_then(|link| {
        world
            .get::<nightshade::ecs::transform::components::LocalTransform>(link.body)
            .map(|transform| transform.translation)
    });

    if let Some(hips_position) = hips_position {
        let feet = hips_position - Vec3::new(0.0, HIPS_FEET_DROP, 0.0);
        if let Some(transform) =
            world.get_mut::<nightshade::ecs::transform::components::LocalTransform>(body)
        {
            transform.translation = feet;
        }
    }

    let mut world_rotation_by_bone: HashMap<Entity, nalgebra_glm::Quat> = HashMap::new();
    for link in &sandbox_world.ragdoll.links {
        if let Some(transform) =
            world.get::<nightshade::ecs::transform::components::LocalTransform>(link.body)
        {
            world_rotation_by_bone.insert(link.bone, transform.rotation * link.rotation_offset);
        }
    }

    #[derive(Clone)]
    struct StackPose {
        rotation: nalgebra_glm::Quat,
        position: Vec3,
        scale: Vec3,
    }
    let body_pose = world
        .get::<nightshade::ecs::transform::components::LocalTransform>(body)
        .map(|transform| StackPose {
            rotation: transform.rotation,
            position: transform.translation,
            scale: transform.scale,
        });
    let Some(body_pose) = body_pose else {
        return;
    };

    let mut writes: Vec<(Entity, nalgebra_glm::Quat, Option<Vec3>)> = Vec::new();
    let mut stack = vec![(model, body_pose)];
    while let Some((entity, parent)) = stack.pop() {
        let Some(local) = world
            .get::<nightshade::ecs::transform::components::LocalTransform>(entity)
            .map(|transform| (transform.rotation, transform.translation, transform.scale))
        else {
            continue;
        };
        let (mut local_rotation, mut local_translation, local_scale) = local;
        if let Some(&world_rotation) = world_rotation_by_bone.get(&entity) {
            local_rotation = nalgebra_glm::quat_normalize(
                &(nalgebra_glm::quat_inverse(&parent.rotation) * world_rotation),
            );
            let mut new_translation = None;
            if Some(entity) == hips_bone
                && let Some(hips_position) = hips_position
            {
                let delta = hips_position - parent.position;
                let rotated = nalgebra_glm::quat_rotate_vec3(
                    &nalgebra_glm::quat_inverse(&parent.rotation),
                    &delta,
                );
                local_translation = rotated.component_div(&parent.scale);
                new_translation = Some(local_translation);
            }
            writes.push((entity, local_rotation, new_translation));
        }
        let child_pose = StackPose {
            position: parent.position
                + nalgebra_glm::quat_rotate_vec3(
                    &parent.rotation,
                    &parent.scale.component_mul(&local_translation),
                ),
            rotation: parent.rotation * local_rotation,
            scale: parent.scale.component_mul(&local_scale),
        };
        for child in query_children(world, entity) {
            stack.push((child, child_pose.clone()));
        }
    }

    for (entity, rotation, translation) in writes {
        if let Some(transform) =
            world.get_mut::<nightshade::ecs::transform::components::LocalTransform>(entity)
        {
            transform.rotation = rotation;
            if let Some(translation) = translation {
                transform.translation = translation;
            }
        }
    }
}
