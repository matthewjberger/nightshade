use super::events::{CollisionEvent, CollisionEventCollector, ContactForceEvent};
#[cfg(feature = "debug_render")]
use crate::ecs::lines::components::Line;
use nightshade_ecs::Entity;
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
    pub debug_entity: Option<nightshade_ecs::Entity>,
    pub debug_version: u64,
    #[cfg(feature = "debug_render")]
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
            debug_entity: None,
            debug_version: 0,
            #[cfg(feature = "debug_render")]
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

pub fn physics_world_step(physics: &mut PhysicsWorld) {
    let _span =
        tracing::info_span!("physics_step", colliders = physics.collider_set.len()).entered();

    physics.physics_pipeline.step(
        &physics.gravity,
        &physics.integration_parameters,
        &mut physics.island_manager,
        &mut physics.broad_phase,
        &mut physics.narrow_phase,
        &mut physics.rigid_body_set,
        &mut physics.collider_set,
        &mut physics.impulse_joint_set,
        &mut physics.multibody_joint_set,
        &mut physics.ccd_solver,
        &physics.physics_hooks,
        &physics.event_collector,
    );
}

pub fn physics_world_collision_events(physics: &PhysicsWorld) -> &[CollisionEvent] {
    &physics.collision_events
}

pub fn physics_world_contact_force_events(physics: &PhysicsWorld) -> &[ContactForceEvent] {
    &physics.contact_force_events
}

pub fn physics_world_set_collision_events(physics: &mut PhysicsWorld, events: Vec<CollisionEvent>) {
    physics.collision_events = events;
}

pub fn physics_world_set_contact_force_events(
    physics: &mut PhysicsWorld,
    events: Vec<ContactForceEvent>,
) {
    physics.contact_force_events = events;
}

pub fn physics_world_clear_events(physics: &mut PhysicsWorld) {
    physics.collision_events.clear();
    physics.contact_force_events.clear();
}

pub(crate) fn physics_world_query_pipeline(physics: &PhysicsWorld) -> QueryPipeline<'_> {
    physics.broad_phase.as_query_pipeline(
        physics.narrow_phase.query_dispatcher(),
        &physics.rigid_body_set,
        &physics.collider_set,
        QueryFilter::default(),
    )
}

pub fn physics_world_get_interpolation_alpha(physics: &PhysicsWorld) -> f32 {
    physics.interpolation_alpha
}

pub(crate) fn physics_world_add_rigid_body(
    physics: &mut PhysicsWorld,
    rigid_body: RigidBody,
) -> RigidBodyHandle {
    physics.rigid_body_set.insert(rigid_body)
}

pub(crate) fn physics_world_add_collider(
    physics: &mut PhysicsWorld,
    collider: Collider,
    parent_handle: RigidBodyHandle,
) -> ColliderHandle {
    physics.debug_version = physics.debug_version.wrapping_add(1);
    physics
        .collider_set
        .insert_with_parent(collider, parent_handle, &mut physics.rigid_body_set)
}

pub(crate) fn physics_world_remove_rigid_body(physics: &mut PhysicsWorld, handle: RigidBodyHandle) {
    physics.debug_version = physics.debug_version.wrapping_add(1);
    if let Some(entity) = physics.handle_to_entity.remove(&handle) {
        physics.entity_to_handle.remove(&entity);
    }
    physics.rigid_body_set.remove(
        handle,
        &mut physics.island_manager,
        &mut physics.collider_set,
        &mut physics.impulse_joint_set,
        &mut physics.multibody_joint_set,
        true,
    );
}

pub(crate) fn physics_world_get_rigid_body_mut(
    physics: &mut PhysicsWorld,
    handle: RigidBodyHandle,
) -> Option<&mut RigidBody> {
    physics.rigid_body_set.get_mut(handle)
}

pub fn physics_world_cast_ray(
    physics: &PhysicsWorld,
    origin: nalgebra_glm::Vec3,
    direction: nalgebra_glm::Vec3,
    max_distance: f32,
    exclude: Option<Entity>,
) -> Option<super::types::RaycastHit> {
    let mut filter = QueryFilter::default();
    if let Some(entity) = exclude
        && let Some(handle) = physics.entity_to_handle.get(&entity)
    {
        filter = filter.exclude_rigid_body(*handle);
    }

    let pipeline = physics.broad_phase.as_query_pipeline(
        physics.narrow_phase.query_dispatcher(),
        &physics.rigid_body_set,
        &physics.collider_set,
        filter,
    );

    let ray = Ray::new(
        point![origin.x, origin.y, origin.z],
        vector![direction.x, direction.y, direction.z],
    );

    let (collider_handle, intersection) =
        pipeline.cast_ray_and_get_normal(&ray, max_distance, true)?;

    let collider = physics.collider_set.get(collider_handle)?;
    let parent_handle = collider.parent()?;
    let entity = *physics.handle_to_entity.get(&parent_handle)?;

    let point_world = ray.point_at(intersection.time_of_impact);
    Some(super::types::RaycastHit {
        entity,
        distance: intersection.time_of_impact,
        point: nalgebra_glm::vec3(point_world.x, point_world.y, point_world.z),
        normal: nalgebra_glm::vec3(
            intersection.normal.x,
            intersection.normal.y,
            intersection.normal.z,
        ),
    })
}

pub fn physics_world_overlap_sphere(
    physics: &PhysicsWorld,
    center: nalgebra_glm::Vec3,
    radius: f32,
    exclude: Option<Entity>,
) -> Vec<Entity> {
    let mut filter = QueryFilter::default();
    if let Some(entity) = exclude
        && let Some(handle) = physics.entity_to_handle.get(&entity)
    {
        filter = filter.exclude_rigid_body(*handle);
    }

    let pipeline = physics.broad_phase.as_query_pipeline(
        physics.narrow_phase.query_dispatcher(),
        &physics.rigid_body_set,
        &physics.collider_set,
        filter,
    );

    let shape = Ball::new(radius);
    let pose = rapier3d::na::Isometry3::translation(center.x, center.y, center.z);

    let mut hits = Vec::new();
    for (collider_handle, _collider) in pipeline.intersect_shape(pose, &shape) {
        if let Some(collider) = physics.collider_set.get(collider_handle)
            && let Some(parent_handle) = collider.parent()
            && let Some(&entity) = physics.handle_to_entity.get(&parent_handle)
        {
            hits.push(entity);
        }
    }
    hits
}

/// Exact penetration contacts between a vertical capsule and the world.
/// Pass `half_height` zero to query a sphere.
pub fn physics_world_capsule_contacts(
    physics: &PhysicsWorld,
    center: nalgebra_glm::Vec3,
    half_height: f32,
    radius: f32,
    exclude: Option<Entity>,
) -> Vec<super::types::ShapeContact> {
    let mut filter = QueryFilter::default();
    if let Some(entity) = exclude
        && let Some(handle) = physics.entity_to_handle.get(&entity)
    {
        filter = filter.exclude_rigid_body(*handle);
    }

    let pipeline = physics.broad_phase.as_query_pipeline(
        physics.narrow_phase.query_dispatcher(),
        &physics.rigid_body_set,
        &physics.collider_set,
        filter,
    );

    let shape = Capsule::new_y(half_height, radius);
    let pose = rapier3d::na::Isometry3::translation(center.x, center.y, center.z);

    let mut contacts = Vec::new();
    for (_collider_handle, collider) in pipeline.intersect_shape(pose, &shape) {
        let Ok(Some(contact)) = rapier3d::parry::query::contact(
            &pose,
            &shape,
            collider.position(),
            collider.shape(),
            0.0,
        ) else {
            continue;
        };
        if contact.dist >= 0.0 {
            continue;
        }
        let Some(parent_handle) = collider.parent() else {
            continue;
        };
        let Some(&entity) = physics.handle_to_entity.get(&parent_handle) else {
            continue;
        };
        contacts.push(super::types::ShapeContact {
            entity,
            point: nalgebra_glm::vec3(contact.point1.x, contact.point1.y, contact.point1.z),
            normal: nalgebra_glm::vec3(-contact.normal1.x, -contact.normal1.y, -contact.normal1.z),
            depth: -contact.dist,
        });
    }
    contacts
}

pub fn physics_world_apply_impulse(
    physics: &mut PhysicsWorld,
    entity: Entity,
    impulse: nalgebra_glm::Vec3,
) {
    let Some(handle) = physics.entity_to_handle.get(&entity).copied() else {
        return;
    };
    let Some(rb) = physics.rigid_body_set.get_mut(handle) else {
        return;
    };
    rb.apply_impulse(vector![impulse.x, impulse.y, impulse.z], true);
}

pub fn physics_world_apply_impulse_at_point(
    physics: &mut PhysicsWorld,
    entity: Entity,
    impulse: nalgebra_glm::Vec3,
    point: nalgebra_glm::Vec3,
) {
    let Some(handle) = physics.entity_to_handle.get(&entity).copied() else {
        return;
    };
    let Some(rb) = physics.rigid_body_set.get_mut(handle) else {
        return;
    };
    rb.apply_impulse_at_point(
        vector![impulse.x, impulse.y, impulse.z],
        point![point.x, point.y, point.z],
        true,
    );
}

pub fn physics_world_apply_force(
    physics: &mut PhysicsWorld,
    entity: Entity,
    force: nalgebra_glm::Vec3,
) {
    let Some(handle) = physics.entity_to_handle.get(&entity).copied() else {
        return;
    };
    let Some(rb) = physics.rigid_body_set.get_mut(handle) else {
        return;
    };
    rb.add_force(vector![force.x, force.y, force.z], true);
}

pub fn physics_world_apply_torque(
    physics: &mut PhysicsWorld,
    entity: Entity,
    torque: nalgebra_glm::Vec3,
) {
    let Some(handle) = physics.entity_to_handle.get(&entity).copied() else {
        return;
    };
    let Some(rb) = physics.rigid_body_set.get_mut(handle) else {
        return;
    };
    rb.add_torque(vector![torque.x, torque.y, torque.z], true);
}

pub fn physics_world_linear_velocity(
    physics: &PhysicsWorld,
    entity: Entity,
) -> Option<nalgebra_glm::Vec3> {
    let handle = physics.entity_to_handle.get(&entity)?;
    let rb = physics.rigid_body_set.get(*handle)?;
    let velocity = rb.linvel();
    Some(nalgebra_glm::vec3(velocity.x, velocity.y, velocity.z))
}

pub fn physics_world_set_linear_velocity(
    physics: &mut PhysicsWorld,
    entity: Entity,
    velocity: nalgebra_glm::Vec3,
) {
    let Some(handle) = physics.entity_to_handle.get(&entity).copied() else {
        return;
    };
    let Some(rb) = physics.rigid_body_set.get_mut(handle) else {
        return;
    };
    rb.set_linvel(vector![velocity.x, velocity.y, velocity.z], true);
}

/// Wakes every rigid body. Bodies created while the simulation was paused sit
/// outside the active set until a step touches them, so the transform sync skips
/// them and they appear frozen. Call this when the simulation resumes so every
/// body integrates and writes back from the first frame.
pub fn physics_world_wake_all(physics: &mut PhysicsWorld) {
    for (_, rigid_body) in physics.rigid_body_set.iter_mut() {
        rigid_body.wake_up(true);
    }
}

pub fn physics_world_angular_velocity(
    physics: &PhysicsWorld,
    entity: Entity,
) -> Option<nalgebra_glm::Vec3> {
    let handle = physics.entity_to_handle.get(&entity)?;
    let rb = physics.rigid_body_set.get(*handle)?;
    let velocity = rb.angvel();
    Some(nalgebra_glm::vec3(velocity.x, velocity.y, velocity.z))
}

pub fn physics_world_set_angular_velocity(
    physics: &mut PhysicsWorld,
    entity: Entity,
    velocity: nalgebra_glm::Vec3,
) {
    let Some(handle) = physics.entity_to_handle.get(&entity).copied() else {
        return;
    };
    let Some(rb) = physics.rigid_body_set.get_mut(handle) else {
        return;
    };
    rb.set_angvel(vector![velocity.x, velocity.y, velocity.z], true);
}

pub fn physics_world_set_linear_damping(physics: &mut PhysicsWorld, entity: Entity, damping: f32) {
    let Some(handle) = physics.entity_to_handle.get(&entity).copied() else {
        return;
    };
    if let Some(rb) = physics.rigid_body_set.get_mut(handle) {
        rb.set_linear_damping(damping);
    }
}

pub fn physics_world_set_angular_damping(physics: &mut PhysicsWorld, entity: Entity, damping: f32) {
    let Some(handle) = physics.entity_to_handle.get(&entity).copied() else {
        return;
    };
    if let Some(rb) = physics.rigid_body_set.get_mut(handle) {
        rb.set_angular_damping(damping);
    }
}

pub fn physics_world_set_gravity_scale(physics: &mut PhysicsWorld, entity: Entity, scale: f32) {
    let Some(handle) = physics.entity_to_handle.get(&entity).copied() else {
        return;
    };
    if let Some(rb) = physics.rigid_body_set.get_mut(handle) {
        rb.set_gravity_scale(scale, true);
    }
}

pub fn physics_world_set_additional_mass(physics: &mut PhysicsWorld, entity: Entity, mass: f32) {
    let Some(handle) = physics.entity_to_handle.get(&entity).copied() else {
        return;
    };
    if let Some(rb) = physics.rigid_body_set.get_mut(handle) {
        rb.set_additional_mass(mass, true);
    }
}

pub fn physics_world_set_friction(physics: &mut PhysicsWorld, entity: Entity, friction: f32) {
    let Some(handle) = physics.entity_to_handle.get(&entity).copied() else {
        return;
    };
    let collider_handles: Vec<ColliderHandle> = physics
        .rigid_body_set
        .get(handle)
        .map(|rb| rb.colliders().to_vec())
        .unwrap_or_default();
    for collider_handle in collider_handles {
        if let Some(collider) = physics.collider_set.get_mut(collider_handle) {
            collider.set_friction(friction);
        }
    }
}

pub fn physics_world_set_restitution(physics: &mut PhysicsWorld, entity: Entity, restitution: f32) {
    let Some(handle) = physics.entity_to_handle.get(&entity).copied() else {
        return;
    };
    let collider_handles: Vec<ColliderHandle> = physics
        .rigid_body_set
        .get(handle)
        .map(|rb| rb.colliders().to_vec())
        .unwrap_or_default();
    for collider_handle in collider_handles {
        if let Some(collider) = physics.collider_set.get_mut(collider_handle) {
            collider.set_restitution(restitution);
        }
    }
}

pub fn physics_world_body_translation(
    physics: &PhysicsWorld,
    entity: Entity,
) -> Option<nalgebra_glm::Vec3> {
    let handle = physics.entity_to_handle.get(&entity)?;
    let rb = physics.rigid_body_set.get(*handle)?;
    let translation = rb.translation();
    Some(nalgebra_glm::vec3(
        translation.x,
        translation.y,
        translation.z,
    ))
}

pub fn physics_world_body_rotation(
    physics: &PhysicsWorld,
    entity: Entity,
) -> Option<nalgebra_glm::Quat> {
    let handle = physics.entity_to_handle.get(&entity)?;
    let rb = physics.rigid_body_set.get(*handle)?;
    let rotation = rb.rotation();
    Some(nalgebra_glm::quat(
        rotation.i, rotation.j, rotation.k, rotation.w,
    ))
}
