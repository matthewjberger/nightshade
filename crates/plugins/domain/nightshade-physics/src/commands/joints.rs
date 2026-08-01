use crate::resources::PhysicsWorld;
use nalgebra_glm;
use nalgebra_glm::Vec3;
use nightshade::ecs::Entity;
use rapier3d::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum JointAxisDirection {
    X,
    Y,
    Z,
}

impl JointAxisDirection {
    pub(crate) fn to_unit_vector(self) -> UnitVector<Real> {
        match self {
            JointAxisDirection::X => Vector::x_axis(),
            JointAxisDirection::Y => Vector::y_axis(),
            JointAxisDirection::Z => Vector::z_axis(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct JointMotor {
    pub target_velocity: f32,
    pub target_position: f32,
    pub stiffness: f32,
    pub damping: f32,
    pub max_force: f32,
}

impl Default for JointMotor {
    fn default() -> Self {
        Self {
            target_velocity: 0.0,
            target_position: 0.0,
            stiffness: 0.0,
            damping: 0.0,
            max_force: f32::MAX,
        }
    }
}

impl JointMotor {
    pub fn velocity(target_velocity: f32, damping: f32) -> Self {
        Self {
            target_velocity,
            target_position: 0.0,
            stiffness: 0.0,
            damping,
            max_force: f32::MAX,
        }
    }

    pub fn position(target_position: f32, stiffness: f32, damping: f32) -> Self {
        Self {
            target_velocity: 0.0,
            target_position,
            stiffness,
            damping,
            max_force: f32::MAX,
        }
    }

    pub fn with_max_force(mut self, max_force: f32) -> Self {
        self.max_force = max_force;
        self
    }
}

#[derive(Debug, Clone, Copy)]
pub struct JointLimits {
    pub min: f32,
    pub max: f32,
}

impl JointLimits {
    pub fn new(min: f32, max: f32) -> Self {
        Self { min, max }
    }

    pub fn symmetric(value: f32) -> Self {
        Self {
            min: -value,
            max: value,
        }
    }
}

pub struct FixedJoint {
    pub local_anchor1: Vec3,
    pub local_anchor2: Vec3,
    pub local_basis1: nalgebra_glm::Quat,
    pub local_basis2: nalgebra_glm::Quat,
}

impl Default for FixedJoint {
    fn default() -> Self {
        Self {
            local_anchor1: Vec3::zeros(),
            local_anchor2: Vec3::zeros(),
            local_basis1: nalgebra_glm::quat_identity(),
            local_basis2: nalgebra_glm::quat_identity(),
        }
    }
}

impl FixedJoint {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_local_anchor1(mut self, anchor: Vec3) -> Self {
        self.local_anchor1 = anchor;
        self
    }

    pub fn with_local_anchor2(mut self, anchor: Vec3) -> Self {
        self.local_anchor2 = anchor;
        self
    }

    pub fn with_local_basis1(mut self, basis: nalgebra_glm::Quat) -> Self {
        self.local_basis1 = basis;
        self
    }

    pub fn with_local_basis2(mut self, basis: nalgebra_glm::Quat) -> Self {
        self.local_basis2 = basis;
        self
    }

    pub(crate) fn build(&self) -> GenericJoint {
        let frame1 = rapier3d::na::Isometry3::from_parts(
            rapier3d::na::Translation3::new(
                self.local_anchor1.x,
                self.local_anchor1.y,
                self.local_anchor1.z,
            ),
            rapier3d::na::UnitQuaternion::new_normalize(rapier3d::na::Quaternion::new(
                self.local_basis1.w,
                self.local_basis1.i,
                self.local_basis1.j,
                self.local_basis1.k,
            )),
        );
        let frame2 = rapier3d::na::Isometry3::from_parts(
            rapier3d::na::Translation3::new(
                self.local_anchor2.x,
                self.local_anchor2.y,
                self.local_anchor2.z,
            ),
            rapier3d::na::UnitQuaternion::new_normalize(rapier3d::na::Quaternion::new(
                self.local_basis2.w,
                self.local_basis2.i,
                self.local_basis2.j,
                self.local_basis2.k,
            )),
        );
        FixedJointBuilder::new()
            .local_frame1(frame1)
            .local_frame2(frame2)
            .build()
            .into()
    }
}

pub struct SphericalJoint {
    pub local_anchor1: Vec3,
    pub local_anchor2: Vec3,
}

impl Default for SphericalJoint {
    fn default() -> Self {
        Self {
            local_anchor1: Vec3::zeros(),
            local_anchor2: Vec3::zeros(),
        }
    }
}

impl SphericalJoint {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_local_anchor1(mut self, anchor: Vec3) -> Self {
        self.local_anchor1 = anchor;
        self
    }

    pub fn with_local_anchor2(mut self, anchor: Vec3) -> Self {
        self.local_anchor2 = anchor;
        self
    }

    pub(crate) fn build(&self) -> GenericJoint {
        SphericalJointBuilder::new()
            .local_anchor1(point![
                self.local_anchor1.x,
                self.local_anchor1.y,
                self.local_anchor1.z
            ])
            .local_anchor2(point![
                self.local_anchor2.x,
                self.local_anchor2.y,
                self.local_anchor2.z
            ])
            .build()
            .into()
    }
}

pub struct RevoluteJoint {
    pub local_anchor1: Vec3,
    pub local_anchor2: Vec3,
    pub axis: JointAxisDirection,
    pub limits: Option<JointLimits>,
    pub motor: Option<JointMotor>,
}

impl Default for RevoluteJoint {
    fn default() -> Self {
        Self {
            local_anchor1: Vec3::zeros(),
            local_anchor2: Vec3::zeros(),
            axis: JointAxisDirection::Y,
            limits: None,
            motor: None,
        }
    }
}

impl RevoluteJoint {
    pub fn new(axis: JointAxisDirection) -> Self {
        Self {
            axis,
            ..Default::default()
        }
    }

    pub fn with_local_anchor1(mut self, anchor: Vec3) -> Self {
        self.local_anchor1 = anchor;
        self
    }

    pub fn with_local_anchor2(mut self, anchor: Vec3) -> Self {
        self.local_anchor2 = anchor;
        self
    }

    pub fn with_limits(mut self, limits: JointLimits) -> Self {
        self.limits = Some(limits);
        self
    }

    pub fn with_motor(mut self, motor: JointMotor) -> Self {
        self.motor = Some(motor);
        self
    }

    pub(crate) fn build(&self) -> GenericJoint {
        let mut builder = RevoluteJointBuilder::new(self.axis.to_unit_vector())
            .local_anchor1(point![
                self.local_anchor1.x,
                self.local_anchor1.y,
                self.local_anchor1.z
            ])
            .local_anchor2(point![
                self.local_anchor2.x,
                self.local_anchor2.y,
                self.local_anchor2.z
            ]);

        if let Some(limits) = self.limits {
            builder = builder.limits([limits.min, limits.max]);
        }

        if let Some(motor) = self.motor {
            builder = builder.motor_velocity(motor.target_velocity, motor.damping);
            if motor.max_force < f32::MAX {
                builder = builder.motor_max_force(motor.max_force);
            }
        }

        builder.build().into()
    }
}

pub struct PrismaticJoint {
    pub local_anchor1: Vec3,
    pub local_anchor2: Vec3,
    pub axis: JointAxisDirection,
    pub limits: Option<JointLimits>,
    pub motor: Option<JointMotor>,
}

impl Default for PrismaticJoint {
    fn default() -> Self {
        Self {
            local_anchor1: Vec3::zeros(),
            local_anchor2: Vec3::zeros(),
            axis: JointAxisDirection::X,
            limits: None,
            motor: None,
        }
    }
}

impl PrismaticJoint {
    pub fn new(axis: JointAxisDirection) -> Self {
        Self {
            axis,
            ..Default::default()
        }
    }

    pub fn with_local_anchor1(mut self, anchor: Vec3) -> Self {
        self.local_anchor1 = anchor;
        self
    }

    pub fn with_local_anchor2(mut self, anchor: Vec3) -> Self {
        self.local_anchor2 = anchor;
        self
    }

    pub fn with_limits(mut self, limits: JointLimits) -> Self {
        self.limits = Some(limits);
        self
    }

    pub fn with_motor(mut self, motor: JointMotor) -> Self {
        self.motor = Some(motor);
        self
    }

    pub(crate) fn build(&self) -> GenericJoint {
        let mut builder = PrismaticJointBuilder::new(self.axis.to_unit_vector())
            .local_anchor1(point![
                self.local_anchor1.x,
                self.local_anchor1.y,
                self.local_anchor1.z
            ])
            .local_anchor2(point![
                self.local_anchor2.x,
                self.local_anchor2.y,
                self.local_anchor2.z
            ]);

        if let Some(limits) = self.limits {
            builder = builder.limits([limits.min, limits.max]);
        }

        if let Some(motor) = self.motor {
            builder = builder.motor_velocity(motor.target_velocity, motor.damping);
            if motor.max_force < f32::MAX {
                builder = builder.motor_max_force(motor.max_force);
            }
        }

        builder.build().into()
    }
}

pub struct RopeJoint {
    pub local_anchor1: Vec3,
    pub local_anchor2: Vec3,
    pub max_distance: f32,
}

impl Default for RopeJoint {
    fn default() -> Self {
        Self {
            local_anchor1: Vec3::zeros(),
            local_anchor2: Vec3::zeros(),
            max_distance: 1.0,
        }
    }
}

impl RopeJoint {
    pub fn new(max_distance: f32) -> Self {
        Self {
            max_distance,
            ..Default::default()
        }
    }

    pub fn with_local_anchor1(mut self, anchor: Vec3) -> Self {
        self.local_anchor1 = anchor;
        self
    }

    pub fn with_local_anchor2(mut self, anchor: Vec3) -> Self {
        self.local_anchor2 = anchor;
        self
    }

    pub(crate) fn build(&self) -> GenericJoint {
        RopeJointBuilder::new(self.max_distance)
            .local_anchor1(point![
                self.local_anchor1.x,
                self.local_anchor1.y,
                self.local_anchor1.z
            ])
            .local_anchor2(point![
                self.local_anchor2.x,
                self.local_anchor2.y,
                self.local_anchor2.z
            ])
            .build()
            .into()
    }
}

pub struct SpringJoint {
    pub local_anchor1: Vec3,
    pub local_anchor2: Vec3,
    pub rest_length: f32,
    pub stiffness: f32,
    pub damping: f32,
}

impl Default for SpringJoint {
    fn default() -> Self {
        Self {
            local_anchor1: Vec3::zeros(),
            local_anchor2: Vec3::zeros(),
            rest_length: 1.0,
            stiffness: 100.0,
            damping: 1.0,
        }
    }
}

impl SpringJoint {
    pub fn new(rest_length: f32, stiffness: f32, damping: f32) -> Self {
        Self {
            rest_length,
            stiffness,
            damping,
            ..Default::default()
        }
    }

    pub fn with_local_anchor1(mut self, anchor: Vec3) -> Self {
        self.local_anchor1 = anchor;
        self
    }

    pub fn with_local_anchor2(mut self, anchor: Vec3) -> Self {
        self.local_anchor2 = anchor;
        self
    }

    pub(crate) fn build(&self) -> GenericJoint {
        SpringJointBuilder::new(self.rest_length, self.stiffness, self.damping)
            .local_anchor1(point![
                self.local_anchor1.x,
                self.local_anchor1.y,
                self.local_anchor1.z
            ])
            .local_anchor2(point![
                self.local_anchor2.x,
                self.local_anchor2.y,
                self.local_anchor2.z
            ])
            .build()
            .into()
    }
}

pub struct JointHandle(pub u64);

pub(crate) fn joint_handle_from_rapier(value: ImpulseJointHandle) -> JointHandle {
    let (index, generation) = value.into_raw_parts();
    JointHandle(((generation as u64) << 32) | (index as u64))
}

pub fn create_fixed_joint(
    storage: &nightshade::ecs::Storage,
    physics: &mut PhysicsWorld,
    parent_entity: Entity,
    child_entity: Entity,
    joint: FixedJoint,
) -> Option<JointHandle> {
    create_joint_internal(
        storage,
        physics,
        parent_entity,
        child_entity,
        joint.build(),
        true,
    )
}

pub fn create_spherical_joint(
    storage: &nightshade::ecs::Storage,
    physics: &mut PhysicsWorld,
    parent_entity: Entity,
    child_entity: Entity,
    joint: SphericalJoint,
) -> Option<JointHandle> {
    create_joint_internal(
        storage,
        physics,
        parent_entity,
        child_entity,
        joint.build(),
        true,
    )
}

pub fn create_revolute_joint(
    storage: &nightshade::ecs::Storage,
    physics: &mut PhysicsWorld,
    parent_entity: Entity,
    child_entity: Entity,
    joint: RevoluteJoint,
) -> Option<JointHandle> {
    create_joint_internal(
        storage,
        physics,
        parent_entity,
        child_entity,
        joint.build(),
        true,
    )
}

pub fn create_prismatic_joint(
    storage: &nightshade::ecs::Storage,
    physics: &mut PhysicsWorld,
    parent_entity: Entity,
    child_entity: Entity,
    joint: PrismaticJoint,
) -> Option<JointHandle> {
    create_joint_internal(
        storage,
        physics,
        parent_entity,
        child_entity,
        joint.build(),
        true,
    )
}

pub fn create_rope_joint(
    storage: &nightshade::ecs::Storage,
    physics: &mut PhysicsWorld,
    parent_entity: Entity,
    child_entity: Entity,
    joint: RopeJoint,
) -> Option<JointHandle> {
    create_joint_internal(
        storage,
        physics,
        parent_entity,
        child_entity,
        joint.build(),
        true,
    )
}

pub fn create_spring_joint(
    storage: &nightshade::ecs::Storage,
    physics: &mut PhysicsWorld,
    parent_entity: Entity,
    child_entity: Entity,
    joint: SpringJoint,
) -> Option<JointHandle> {
    create_joint_internal(
        storage,
        physics,
        parent_entity,
        child_entity,
        joint.build(),
        true,
    )
}

fn create_joint_internal(
    storage: &nightshade::ecs::Storage,
    physics: &mut PhysicsWorld,
    parent_entity: Entity,
    child_entity: Entity,
    joint: GenericJoint,
    collisions_enabled: bool,
) -> Option<JointHandle> {
    let parent_handle = storage
        .get::<crate::contract::components::RigidBodyComponent>(parent_entity)
        .and_then(|rb| rb.handle);
    let child_handle = storage
        .get::<crate::contract::components::RigidBodyComponent>(child_entity)
        .and_then(|rb| rb.handle);

    match (parent_handle, child_handle) {
        (Some(parent_h), Some(child_h)) => {
            let joint_handle = physics.impulse_joint_set.insert(
                crate::data::rapier_convert::rigid_body_handle_to_rapier(parent_h),
                crate::data::rapier_convert::rigid_body_handle_to_rapier(child_h),
                joint,
                collisions_enabled,
            );
            Some(joint_handle_from_rapier(joint_handle))
        }
        _ => {
            physics.pending_joints.push(crate::resources::PendingJoint {
                parent_entity,
                child_entity,
                joint,
                joint_type: crate::resources::JointType::Fixed,
                collisions_enabled,
                spring_rest_length: None,
                spring_stiffness: None,
                spring_damping: None,
            });
            None
        }
    }
}
