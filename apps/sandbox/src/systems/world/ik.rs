use crate::ecs::{IkChain, SandboxResources};
use nightshade::prelude::*;

const ANKLE_CLEARANCE: f32 = 0.09;
const FOOT_PROBE_RISE: f32 = 0.6;
const FOOT_PROBE_DISTANCE: f32 = 1.4;
const HAND_EDGE_FORWARD: f32 = 0.5;
const HAND_EDGE_SPREAD: f32 = 0.22;
const HAND_EDGE_LIFT: f32 = 0.03;

pub fn update(sandbox_world: &SandboxResources, world: &mut World) {
    if sandbox_world.ragdoll.mode != crate::ecs::RagdollMode::Off {
        return;
    }
    let character = &sandbox_world.character;
    if !character.third_person {
        return;
    }
    let Some(body) = character.body else {
        return;
    };
    if character.model.is_none() {
        return;
    }

    if character.hanging {
        solve_hands_to_ledge(sandbox_world, world, body);
        return;
    }

    if character.tuck_weight > 0.3 {
        return;
    }
    for chain in sandbox_world.pose.leg_chains.clone() {
        solve_foot_to_ground(world, body, chain);
    }
}

/// Push a penetrating foot up to the ground surface, bending the knee
/// and swinging the hip with the analytic two-bone solution. Feet are
/// only ever raised, never pulled down, so swing legs stay authored.
fn solve_foot_to_ground(world: &mut World, body: Entity, chain: IkChain) {
    let Some(ankle_position) = world_pose_of(world, body, chain.end).map(|pose| pose.position)
    else {
        return;
    };
    let probe_origin = ankle_position + Vec3::new(0.0, FOOT_PROBE_RISE, 0.0);
    let Some(hit) = physics_world_cast_ray(
        world.plugin_resource::<PhysicsWorld>(),
        probe_origin,
        Vec3::new(0.0, -1.0, 0.0),
        FOOT_PROBE_DISTANCE,
        None,
    ) else {
        return;
    };
    let target_y = hit.point.y + ANKLE_CLEARANCE;
    if ankle_position.y >= target_y {
        return;
    }
    let target = Vec3::new(ankle_position.x, target_y, ankle_position.z);
    solve_two_bone(world, body, chain, target);
}

fn solve_hands_to_ledge(sandbox_world: &SandboxResources, world: &mut World, body: Entity) {
    let character = &sandbox_world.character;
    let Some(feet) = world
        .get::<nightshade::ecs::transform::components::LocalTransform>(body)
        .map(|transform| transform.translation)
    else {
        return;
    };
    let wall_direction = -character.hang_normal;
    let hand_height = character.hang_edge_y + HAND_EDGE_LIFT;
    let hand_center = feet + wall_direction * HAND_EDGE_FORWARD;
    let arm_chains = sandbox_world.pose.arm_chains.clone();
    for (chain_index, chain) in arm_chains.into_iter().enumerate() {
        let side = if chain_index == 0 { -1.0 } else { 1.0 };
        let target = Vec3::new(
            hand_center.x + character.hang_tangent.x * HAND_EDGE_SPREAD * side,
            hand_height,
            hand_center.z + character.hang_tangent.z * HAND_EDGE_SPREAD * side,
        );
        solve_two_bone(world, body, chain, target);
    }
}

pub struct WorldPose {
    pub rotation: nalgebra_glm::Quat,
    pub position: Vec3,
    pub scale: Vec3,
}

fn compose(parent: &WorldPose, entity: Entity, world: &World) -> Option<WorldPose> {
    let local = world.get::<nightshade::ecs::transform::components::LocalTransform>(entity)?;
    Some(WorldPose {
        position: parent.position
            + nalgebra_glm::quat_rotate_vec3(
                &parent.rotation,
                &parent.scale.component_mul(&local.translation),
            ),
        rotation: parent.rotation * local.rotation,
        scale: parent.scale.component_mul(&local.scale),
    })
}

fn ancestry(world: &World, body: Entity, entity: Entity) -> Vec<Entity> {
    let mut chain = vec![entity];
    let mut current = entity;
    while let Some(parent) = world
        .get::<freecs::dynamic::ChildOf>(current)
        .map(|child_of| child_of.0)
    {
        chain.push(parent);
        if parent == body {
            break;
        }
        current = parent;
    }
    chain.reverse();
    chain
}

pub fn world_pose_of(world: &World, body: Entity, entity: Entity) -> Option<WorldPose> {
    let path = ancestry(world, body, entity);
    if path.first() != Some(&body) {
        return None;
    }
    let mut pose = WorldPose {
        rotation: nalgebra_glm::quat_identity(),
        position: Vec3::zeros(),
        scale: Vec3::new(1.0, 1.0, 1.0),
    };
    for &step in &path {
        pose = compose(&pose, step, world)?;
    }
    Some(pose)
}

/// Analytic two-bone inverse kinematics: set the hinge angle from the
/// law of cosines, then swing the root so the end effector lands on
/// the target, preserving the end joint's world orientation.
fn solve_two_bone(world: &mut World, body: Entity, chain: IkChain, target: Vec3) {
    let path = ancestry(world, body, chain.end);
    if path.first() != Some(&body) {
        return;
    }

    let mut pose = WorldPose {
        rotation: nalgebra_glm::quat_identity(),
        position: Vec3::zeros(),
        scale: Vec3::new(1.0, 1.0, 1.0),
    };
    let mut root_parent_rotation = nalgebra_glm::quat_identity();
    let mut root_pose = None;
    let mut middle_pose = None;
    let mut end_pose = None;
    for &step in &path {
        if step == chain.root {
            root_parent_rotation = pose.rotation;
        }
        let Some(next) = compose(&pose, step, world) else {
            return;
        };
        pose = next;
        if step == chain.root {
            root_pose = Some(WorldPose {
                rotation: pose.rotation,
                position: pose.position,
                scale: pose.scale,
            });
        } else if step == chain.middle {
            middle_pose = Some(WorldPose {
                rotation: pose.rotation,
                position: pose.position,
                scale: pose.scale,
            });
        } else if step == chain.end {
            end_pose = Some(WorldPose {
                rotation: pose.rotation,
                position: pose.position,
                scale: pose.scale,
            });
        }
    }
    let (Some(root_pose), Some(middle_pose), Some(end_pose)) = (root_pose, middle_pose, end_pose)
    else {
        return;
    };

    let hip = root_pose.position;
    let knee = middle_pose.position;
    let ankle = end_pose.position;
    let upper_length = (knee - hip).norm();
    let lower_length = (ankle - knee).norm();
    if upper_length < 1e-4 || lower_length < 1e-4 {
        return;
    }

    let reach = (target - hip).norm().clamp(
        (upper_length - lower_length).abs() + 0.01,
        (upper_length + lower_length) * 0.999,
    );
    let target_direction = (target - hip).normalize();
    let clamped_target = hip + target_direction * reach;

    let desired_cosine = ((upper_length * upper_length + lower_length * lower_length
        - reach * reach)
        / (2.0 * upper_length * lower_length))
        .clamp(-1.0, 1.0);
    let desired_angle = desired_cosine.acos();
    let to_hip = (hip - knee).normalize();
    let to_ankle = (ankle - knee).normalize();
    let current_angle = to_hip.dot(&to_ankle).clamp(-1.0, 1.0).acos();

    let mut hinge = nalgebra_glm::cross(&to_hip, &to_ankle);
    if hinge.norm() < 1e-4 {
        hinge = nalgebra_glm::quat_rotate_vec3(&middle_pose.rotation, &Vec3::x());
    }
    let hinge = hinge.normalize();
    let middle_delta = nalgebra_glm::quat_angle_axis(desired_angle - current_angle, &hinge);
    let bent_ankle = knee + nalgebra_glm::quat_rotate_vec3(&middle_delta, &(ankle - knee));

    let root_delta = rotation_between(bent_ankle - hip, clamped_target - hip);

    let new_root_local = nalgebra_glm::quat_inverse(&root_parent_rotation)
        * root_delta
        * root_parent_rotation
        * world
            .get::<nightshade::ecs::transform::components::LocalTransform>(chain.root)
            .map(|transform| transform.rotation)
            .unwrap_or_else(nalgebra_glm::quat_identity);
    let new_middle_local = nalgebra_glm::quat_inverse(&root_pose.rotation)
        * middle_delta
        * root_pose.rotation
        * world
            .get::<nightshade::ecs::transform::components::LocalTransform>(chain.middle)
            .map(|transform| transform.rotation)
            .unwrap_or_else(nalgebra_glm::quat_identity);
    let new_middle_world = root_delta * middle_delta * middle_pose.rotation;
    let new_end_local = nalgebra_glm::quat_inverse(&new_middle_world) * end_pose.rotation;

    if let Some(transform) =
        world.get_mut::<nightshade::ecs::transform::components::LocalTransform>(chain.root)
    {
        transform.rotation = nalgebra_glm::quat_normalize(&new_root_local);
    }
    if let Some(transform) =
        world.get_mut::<nightshade::ecs::transform::components::LocalTransform>(chain.middle)
    {
        transform.rotation = nalgebra_glm::quat_normalize(&new_middle_local);
    }
    if let Some(transform) =
        world.get_mut::<nightshade::ecs::transform::components::LocalTransform>(chain.end)
    {
        transform.rotation = nalgebra_glm::quat_normalize(&new_end_local);
    }
}

pub fn rotation_between(from: Vec3, to: Vec3) -> nalgebra_glm::Quat {
    let from_length = from.norm();
    let to_length = to.norm();
    if from_length < 1e-5 || to_length < 1e-5 {
        return nalgebra_glm::quat_identity();
    }
    let from_unit = from / from_length;
    let to_unit = to / to_length;
    let cosine = from_unit.dot(&to_unit).clamp(-1.0, 1.0);
    let axis = nalgebra_glm::cross(&from_unit, &to_unit);
    if axis.norm() < 1e-6 {
        if cosine > 0.0 {
            return nalgebra_glm::quat_identity();
        }
        let fallback = if from_unit.x.abs() < 0.9 {
            Vec3::x()
        } else {
            Vec3::y()
        };
        let perpendicular = nalgebra_glm::cross(&from_unit, &fallback).normalize();
        return nalgebra_glm::quat_angle_axis(std::f32::consts::PI, &perpendicular);
    }
    nalgebra_glm::quat_angle_axis(cosine.acos(), &axis.normalize())
}
