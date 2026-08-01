pub mod events;
mod picking;
use crate::resources::events::{CollisionEvent, CollisionEventCollector, ContactForceEvent};
use nalgebra_glm;
use nightshade::ecs::Entity;
use nightshade_render::components::lines::Line;
pub use picking::*;
use rapier3d::prelude::*;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum JointType {
    Fixed,
    Revolute,
    Prismatic,
    Spherical,
    Rope,
    Spring,
}

pub struct JointInfo {
    pub parent_entity: Entity,
    pub child_entity: Entity,
    pub joint_type: JointType,
    pub collisions_enabled: bool,
    pub spring_rest_length: Option<f32>,
    pub spring_stiffness: Option<f32>,
    pub spring_damping: Option<f32>,
}

pub struct PendingJoint {
    pub parent_entity: Entity,
    pub child_entity: Entity,
    pub(crate) joint: GenericJoint,
    pub joint_type: JointType,
    pub collisions_enabled: bool,
    pub spring_rest_length: Option<f32>,
    pub spring_stiffness: Option<f32>,
    pub spring_damping: Option<f32>,
}

pub struct PhysicsWorld {
    pub gravity: nalgebra_glm::Vec3,
    pub(crate) integration_parameters: IntegrationParameters,
    pub(crate) physics_pipeline: PhysicsPipeline,
    pub(crate) island_manager: IslandManager,
    pub(crate) broad_phase: BroadPhaseBvh,
    pub(crate) narrow_phase: NarrowPhase,
    pub(crate) impulse_joint_set: ImpulseJointSet,
    pub(crate) multibody_joint_set: MultibodyJointSet,
    pub(crate) ccd_solver: CCDSolver,
    pub(crate) physics_hooks: (),
    pub(crate) event_collector: CollisionEventCollector,
    pub collision_events: Vec<CollisionEvent>,
    pub contact_force_events: Vec<ContactForceEvent>,
    pub(crate) rigid_body_set: RigidBodySet,
    pub(crate) collider_set: ColliderSet,
    pub fixed_timestep: f32,
    pub accumulator: f32,
    pub max_substeps: u32,
    pub interpolation_alpha: f32,
    pub debug_draw: bool,
    pub debug_version: u64,
    pub(crate) debug_static_lines_cache: HashMap<ColliderHandle, Vec<Line>>,
    pub debug_last_version: u64,
    pub pending_joints: Vec<PendingJoint>,
    pub(crate) joint_registry: HashMap<ImpulseJointHandle, JointInfo>,
    pub(crate) handle_to_entity: HashMap<RigidBodyHandle, Entity>,
    pub(crate) entity_to_handle: HashMap<Entity, RigidBodyHandle>,
    pub enabled: bool,
}

impl Default for PhysicsWorld {
    fn default() -> Self {
        let integration_parameters = IntegrationParameters {
            dt: 1.0 / 60.0,
            ..Default::default()
        };

        Self {
            gravity: nalgebra_glm::vec3(0.0, -9.81, 0.0),
            integration_parameters,
            physics_pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: BroadPhaseBvh::new(),
            narrow_phase: NarrowPhase::new(),
            impulse_joint_set: ImpulseJointSet::new(),
            multibody_joint_set: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
            physics_hooks: (),
            event_collector: CollisionEventCollector::new(),
            collision_events: Vec::new(),
            contact_force_events: Vec::new(),
            rigid_body_set: RigidBodySet::new(),
            collider_set: ColliderSet::new(),
            fixed_timestep: 1.0 / 60.0,
            accumulator: 0.0,
            max_substeps: 4,
            interpolation_alpha: 0.0,
            debug_draw: false,
            debug_version: 0,
            debug_static_lines_cache: HashMap::new(),
            debug_last_version: u64::MAX,
            pending_joints: Vec::new(),
            joint_registry: HashMap::new(),
            handle_to_entity: HashMap::new(),
            entity_to_handle: HashMap::new(),
            enabled: false,
        }
    }
}
