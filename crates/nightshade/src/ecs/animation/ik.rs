//! Inverse kinematics solvers that run on the CPU pose. They read a chain's
//! current world transforms, solve for the joint rotations that place the tip on
//! a target, and write the result back as each bone's local rotation. Call after
//! animation has posed the skeleton for the frame so the solve layers on top.

use nalgebra_glm::{Quat, Vec3};
use serde::{Deserialize, Serialize};

use crate::ecs::world::{Entity, World};

/// A two-bone IK chain authored by bone name and solved on the GPU pose. `target`
/// and optional `pole` are world-space; `weight` blends the solved pose against
/// the animated one. Stored on the [`AnimationPlayer`], one per active constraint.
///
/// [`AnimationPlayer`]: crate::ecs::animation::components::AnimationPlayer
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IkChain {
    /// Name of the chain's root bone (e.g. the upper arm).
    pub root_bone: String,
    /// Name of the middle bone (e.g. the forearm).
    pub mid_bone: String,
    /// Name of the tip bone (e.g. the hand).
    pub tip_bone: String,
    /// World-space goal for the tip.
    pub target: Vec3,
    /// Optional world-space pole that biases the bend direction.
    pub pole: Option<Vec3>,
    /// Blend of the solved pose against the animated pose, 0.0 to 1.0.
    pub weight: f32,
    /// Whether the chain solves this frame.
    pub enabled: bool,
}

/// An N-bone IK chain solved on the GPU with cyclic coordinate descent, for
/// spines, tails, tentacles, and chains longer than two bones. `bones` is
/// ordered root to tip. `max_angle` clamps each per-iteration joint rotation,
/// acting as a stiffness and a soft joint limit.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IkChainMulti {
    /// Bone names ordered from the chain root to the tip.
    pub bones: Vec<String>,
    /// World-space goal for the tip.
    pub target: Vec3,
    /// Blend of the solved pose against the animated pose, 0.0 to 1.0.
    pub weight: f32,
    /// Whether the chain solves this frame.
    pub enabled: bool,
    /// Solver iterations; more iterations reach further and settle tighter.
    pub iterations: u32,
    /// Maximum per-iteration rotation for any joint, in radians.
    pub max_angle: f32,
}

/// One foot in a foot-placement rig. The system raycasts down from the foot's
/// animated position and steers the two-bone IK chain at `ik_chain_index` so the
/// foot plants on the ground, never below the animated pose.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FootDefinition {
    /// Name of the foot (tip) bone, used to find the animated foot position.
    pub tip_bone: String,
    /// Index of the leg's two-bone IK chain in the player's `ik_chains`.
    pub ik_chain_index: usize,
    /// How far above and below the foot to probe for ground.
    pub ray_length: f32,
    /// Height to keep the foot above the ground hit.
    pub foot_height: f32,
}

/// A look-at constraint that aims a chain of bones' forward axis at a world
/// target, distributing the rotation from root to tip so a spine, neck, and head
/// track together. `cone_angle` clamps each bone's deviation. Solved on the GPU.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AimConstraint {
    /// Bone names ordered from the base of the aim chain to the tip that looks.
    pub bones: Vec<String>,
    /// World-space point to look at.
    pub target: Vec3,
    /// The bones' local forward axis that should point at the target.
    pub forward: Vec3,
    /// Blend of the aimed pose against the animated pose, 0.0 to 1.0.
    pub weight: f32,
    /// Whether the constraint solves this frame.
    pub enabled: bool,
    /// Maximum rotation any single bone may take, in radians.
    pub cone_angle: f32,
}

fn world_position(world: &World, entity: Entity) -> Option<Vec3> {
    world
        .get::<crate::ecs::transform::components::GlobalTransform>(entity)
        .map(|transform| {
            Vec3::new(
                transform.0[(0, 3)],
                transform.0[(1, 3)],
                transform.0[(2, 3)],
            )
        })
}

fn world_rotation(world: &World, entity: Entity) -> Option<Quat> {
    world
        .get::<crate::ecs::transform::components::GlobalTransform>(entity)
        .map(|transform| {
            let basis = nalgebra_glm::mat4_to_mat3(&transform.0);
            let x = basis.column(0).normalize();
            let y = basis.column(1).normalize();
            let z = basis.column(2).normalize();
            let orthonormal = nalgebra_glm::Mat3::from_columns(&[x, y, z]);
            nalgebra_glm::mat3_to_quat(&orthonormal).normalize()
        })
}

fn local_rotation(world: &World, entity: Entity) -> Option<Quat> {
    world
        .get::<crate::ecs::transform::components::LocalTransform>(entity)
        .map(|transform| transform.rotation)
}

/// Applies a world-space delta rotation to a bone, preserving everything below
/// it, by converting the delta into the bone's parent space and pre-multiplying
/// the bone's local rotation.
fn apply_world_delta(world: &mut World, entity: Entity, delta: Quat) {
    let Some(bone_world) = world_rotation(world, entity) else {
        return;
    };
    let Some(bone_local) = local_rotation(world, entity) else {
        return;
    };
    let parent_world = bone_world * bone_local.try_inverse().unwrap_or_else(Quat::identity);
    let new_world = (delta * bone_world).normalize();
    let new_local =
        (parent_world.try_inverse().unwrap_or_else(Quat::identity) * new_world).normalize();
    if let Some(transform) =
        world.get_mut::<crate::ecs::transform::components::LocalTransform>(entity)
    {
        transform.rotation = new_local;
    }
}

fn rotation_between(from: &Vec3, to: &Vec3) -> Quat {
    let from = from.normalize();
    let to = to.normalize();
    let dot = from.dot(&to).clamp(-1.0, 1.0);
    if dot > 0.999_999 {
        return Quat::identity();
    }
    if dot < -0.999_999 {
        let mut axis = Vec3::x().cross(&from);
        if axis.magnitude() < 1.0e-4 {
            axis = Vec3::y().cross(&from);
        }
        return nalgebra_glm::quat_angle_axis(std::f32::consts::PI, &axis.normalize());
    }
    let axis = from.cross(&to).normalize();
    let angle = dot.acos();
    nalgebra_glm::quat_angle_axis(angle, &axis)
}

/// Solves a two-bone chain (root, mid, tip) so the tip reaches `target`,
/// analytically. `pole` biases the bend direction of the middle joint, like a
/// knee or elbow hint; pass `None` to keep the current bend plane. Writes new
/// local rotations to `root` and `mid`.
pub fn solve_two_bone_ik(
    world: &mut World,
    root: Entity,
    mid: Entity,
    tip: Entity,
    target: Vec3,
    pole: Option<Vec3>,
) {
    let (Some(a), Some(b), Some(c)) = (
        world_position(world, root),
        world_position(world, mid),
        world_position(world, tip),
    ) else {
        return;
    };

    let length_ab = (b - a).magnitude();
    let length_cb = (c - b).magnitude();
    if length_ab < 1.0e-5 || length_cb < 1.0e-5 {
        return;
    }
    let reach = length_ab + length_cb;
    let to_target = target - a;
    let length_at = to_target.magnitude().clamp(1.0e-4, reach - 1.0e-4);

    let ca = c - a;
    let ba = b - a;
    let cb = c - b;
    let ab = a - b;

    let current_root = angle_between(&ca, &ba);
    let current_mid = angle_between(&ab, &cb);
    let target_root = law_of_cosines(length_ab, length_at, length_cb);
    let target_mid = law_of_cosines(length_ab, length_cb, length_at);

    let bend_axis = match pole {
        Some(pole) => {
            let pole_direction = pole - a;
            let axis = (b - a).cross(&pole_direction);
            if axis.magnitude() < 1.0e-5 {
                ca.cross(&ba)
            } else {
                axis
            }
        }
        None => ca.cross(&ba),
    };
    if bend_axis.magnitude() < 1.0e-5 {
        let aim = rotation_between(&ca, &to_target);
        apply_world_delta(world, root, aim);
        return;
    }
    let bend_axis = bend_axis.normalize();

    let root_delta = nalgebra_glm::quat_angle_axis(target_root - current_root, &bend_axis);
    let mid_delta = nalgebra_glm::quat_angle_axis(target_mid - current_mid, &bend_axis);

    apply_world_delta(world, mid, mid_delta);
    apply_world_delta(world, root, root_delta);

    if let (Some(a), Some(c)) = (world_position(world, root), world_position(world, tip)) {
        let aim = rotation_between(&(c - a), &(target - a));
        apply_world_delta(world, root, aim);
    }
}

/// Orients a single bone so its `forward` axis (in the bone's local space) points
/// at `target` in world space, leaving its position untouched. Use it to aim a
/// head, an eye, or a turret bone.
pub fn look_at_bone(world: &mut World, bone: Entity, target: Vec3, forward: Vec3) {
    let (Some(position), Some(rotation)) =
        (world_position(world, bone), world_rotation(world, bone))
    else {
        return;
    };
    let to_target = target - position;
    if to_target.magnitude() < 1.0e-5 {
        return;
    }
    let current_forward =
        nalgebra_glm::quat_rotate_vec3(&rotation, &forward.normalize()).normalize();
    let delta = rotation_between(&current_forward, &to_target);
    apply_world_delta(world, bone, delta);
}

fn angle_between(a: &Vec3, b: &Vec3) -> f32 {
    let denominator = a.magnitude() * b.magnitude();
    if denominator < 1.0e-6 {
        return 0.0;
    }
    (a.dot(b) / denominator).clamp(-1.0, 1.0).acos()
}

fn law_of_cosines(adjacent_a: f32, adjacent_b: f32, opposite: f32) -> f32 {
    let denominator = 2.0 * adjacent_a * adjacent_b;
    if denominator < 1.0e-6 {
        return 0.0;
    }
    ((adjacent_a * adjacent_a + adjacent_b * adjacent_b - opposite * opposite) / denominator)
        .clamp(-1.0, 1.0)
        .acos()
}
