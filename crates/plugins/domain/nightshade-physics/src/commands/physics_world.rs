use nightshade::ecs::Entity;
use rapier3d::prelude::*;

use crate::resources::PhysicsWorld;
use crate::resources::events::{CollisionEvent, ContactForceEvent};

pub fn physics_world_step(physics: &mut PhysicsWorld) {
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

pub fn physics_world_set_timestep(physics: &mut PhysicsWorld, seconds: f32, substeps: u32) {
    physics.fixed_timestep = seconds;
    physics.integration_parameters.dt = seconds;
    physics.max_substeps = substeps;
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

pub fn physics_world_set_translation(
    physics: &mut PhysicsWorld,
    entity: Entity,
    translation: nalgebra_glm::Vec3,
) {
    let Some(handle) = physics.entity_to_handle.get(&entity).copied() else {
        return;
    };
    let Some(rb) = physics.rigid_body_set.get_mut(handle) else {
        return;
    };
    rb.set_translation(vector![translation.x, translation.y, translation.z], true);
}

pub fn physics_world_wake_all(physics: &mut PhysicsWorld) {
    for (_, rigid_body) in physics.rigid_body_set.iter_mut() {
        rigid_body.wake_up(true);
    }
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
