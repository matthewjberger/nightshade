use super::events::{CollisionEvent, CollisionEventKind, ContactForceEvent};
use super::resources::{
    PhysicsWorld, physics_world_add_collider, physics_world_add_rigid_body,
    physics_world_clear_events, physics_world_get_rigid_body_mut, physics_world_remove_rigid_body,
    physics_world_set_collision_events, physics_world_set_contact_force_events, physics_world_step,
};
use crate::ecs::world::World;
use rapier3d::na::{Quaternion, UnitQuaternion};
use rapier3d::prelude::*;

use crate::prelude::*;
pub fn run_physics_systems(world: &mut World) {
    let rigid_body_count = world.plugin_resource::<PhysicsWorld>().rigid_body_set.len();
    let _span = tracing::info_span!("physics", rigid_bodies = rigid_body_count).entered();

    physics_world_clear_events(world.plugin_resource_mut::<PhysicsWorld>());

    initialize_physics_bodies_system(world);
    sync_collision_listener_system(world);
    process_pending_joints_system(world);

    if !world.plugin_resource::<PhysicsWorld>().enabled {
        return;
    }

    let fixed_timestep = world.plugin_resource::<PhysicsWorld>().fixed_timestep;
    let max_substeps = world.plugin_resource::<PhysicsWorld>().max_substeps;
    let max_frame_time = max_substeps as f32 * fixed_timestep;
    let frame_time = world
        .expect_resource_mut::<crate::ecs::window::resources::Window>()
        .timing
        .delta_time
        .min(max_frame_time);

    world.plugin_resource_mut::<PhysicsWorld>().accumulator += frame_time;

    let mut substeps_performed = 0;

    while world.plugin_resource::<PhysicsWorld>().accumulator >= fixed_timestep
        && substeps_performed < max_substeps
    {
        advance_interpolation_previous_and_restore_transform(world);

        sync_transforms_to_physics_system(world);

        super::character_controller::character_controller_input_system(world);
        super::character_controller::character_controller_system(world);

        physics_world_step(world.plugin_resource_mut::<PhysicsWorld>());

        sync_interpolation_current_from_physics(world);

        world.plugin_resource_mut::<PhysicsWorld>().accumulator -= fixed_timestep;
        substeps_performed += 1;
    }

    drain_collision_events(world);

    tracing::trace!(substeps = substeps_performed);

    let interpolation_alpha =
        (world.plugin_resource::<PhysicsWorld>().accumulator / fixed_timestep).clamp(0.0, 1.0);
    world
        .plugin_resource_mut::<PhysicsWorld>()
        .interpolation_alpha = interpolation_alpha;

    sync_transforms_from_physics_system(world);
    physics_interpolation_system(world);

    cleanup_physics_bodies_system(world);

    #[cfg(feature = "debug_render")]
    super::debug::physics_debug_draw_system(world);
}

fn advance_interpolation_previous_and_restore_transform(world: &mut World) {
    let entities: Vec<_> = world.ecs.worlds[CORE]
        .query_entities(crate::ecs::world::PHYSICS_INTERPOLATION)
        .collect();

    for entity in entities {
        let body_type = world
            .plugin_resource::<PhysicsWorld>()
            .entity_to_handle
            .get(&entity)
            .and_then(|handle| {
                world
                    .plugin_resource::<PhysicsWorld>()
                    .rigid_body_set
                    .get(*handle)
            })
            .map(|rb| rb.body_type());

        let kinematic = matches!(
            body_type,
            Some(RigidBodyType::KinematicPositionBased | RigidBodyType::KinematicVelocityBased)
        );

        let restore = if let Some(interpolation) =
            world.get_mut::<crate::plugins::physics::components::PhysicsInterpolation>(entity)
        {
            interpolation.update_previous();
            if kinematic {
                None
            } else {
                interpolation.enabled.then_some((
                    interpolation.current_translation,
                    interpolation.current_rotation,
                ))
            }
        } else {
            None
        };

        if let Some((translation, rotation)) = restore
            && let Some(transform) =
                world.get_mut::<crate::ecs::transform::components::LocalTransform>(entity)
        {
            transform.translation = translation;
            transform.rotation = rotation;
        }
    }
}

fn sync_interpolation_current_from_physics(world: &mut World) {
    let active_count = world
        .plugin_resource::<PhysicsWorld>()
        .island_manager
        .active_bodies()
        .len();
    let mut updates = Vec::with_capacity(active_count);
    for handle in world
        .plugin_resource::<PhysicsWorld>()
        .island_manager
        .active_bodies()
    {
        let Some(&entity) = world
            .plugin_resource::<PhysicsWorld>()
            .handle_to_entity
            .get(handle)
        else {
            continue;
        };
        let Some(rb) = world
            .plugin_resource::<PhysicsWorld>()
            .rigid_body_set
            .get(*handle)
        else {
            continue;
        };
        let translation = rb.translation();
        let rotation = rb.rotation();
        let new_translation = nalgebra_glm::vec3(translation.x, translation.y, translation.z);
        let new_rotation = nalgebra_glm::quat(rotation.i, rotation.j, rotation.k, rotation.w);
        updates.push((entity, new_translation, new_rotation));
    }
    for (entity, translation, rotation) in updates {
        if let Some(interpolation) =
            world.get_mut::<crate::plugins::physics::components::PhysicsInterpolation>(entity)
        {
            interpolation.update_current(translation, rotation);
        }
    }
}

const TRANSFORM_EPSILON: f32 = 1e-6;

fn transforms_approximately_equal(
    old_translation: &nalgebra_glm::Vec3,
    new_translation: &nalgebra_glm::Vec3,
    old_rotation: &nalgebra_glm::Quat,
    new_rotation: &nalgebra_glm::Quat,
) -> bool {
    let translation_diff = nalgebra_glm::distance(old_translation, new_translation);
    if translation_diff > TRANSFORM_EPSILON {
        return false;
    }
    let rotation_dot = nalgebra_glm::quat_dot(old_rotation, new_rotation).abs();
    rotation_dot > 1.0 - TRANSFORM_EPSILON
}

pub fn sync_transforms_from_physics_system(world: &mut World) {
    let active_count = world
        .plugin_resource::<PhysicsWorld>()
        .island_manager
        .active_bodies()
        .len();
    let mut updates = Vec::with_capacity(active_count);

    for handle in world
        .plugin_resource::<PhysicsWorld>()
        .island_manager
        .active_bodies()
    {
        let Some(&entity) = world
            .plugin_resource::<PhysicsWorld>()
            .handle_to_entity
            .get(handle)
        else {
            continue;
        };

        let Some(rb) = world
            .plugin_resource::<PhysicsWorld>()
            .rigid_body_set
            .get(*handle)
        else {
            continue;
        };

        let translation = rb.translation();
        let rotation = rb.rotation();
        let new_translation = nalgebra_glm::vec3(translation.x, translation.y, translation.z);
        let new_rotation = nalgebra_glm::quat(rotation.i, rotation.j, rotation.k, rotation.w);

        let should_update = if let Some(transform) =
            world.get::<crate::ecs::transform::components::LocalTransform>(entity)
        {
            !transforms_approximately_equal(
                &transform.translation,
                &new_translation,
                &transform.rotation,
                &new_rotation,
            )
        } else {
            true
        };

        if should_update {
            updates.push((entity, new_translation, new_rotation));
        }
    }

    for (entity, translation, rotation) in updates {
        if let Some(transform) =
            world.get_mut::<crate::ecs::transform::components::LocalTransform>(entity)
        {
            transform.translation = translation;
            transform.rotation = rotation;
        }
    }
}

pub fn sync_transforms_to_physics_system(world: &mut World) {
    let entities: Vec<_> = world.ecs.worlds[CORE]
        .query_entities(crate::ecs::world::LOCAL_TRANSFORM | crate::ecs::world::RIGID_BODY)
        .collect();

    for entity in entities {
        let (transform_data, handle) = {
            let transform = world.get::<crate::ecs::transform::components::LocalTransform>(entity);
            let rigid_body =
                world.get::<crate::plugins::physics::components::RigidBodyComponent>(entity);

            if let (Some(transform), Some(rigid_body)) = (transform, rigid_body)
                && let Some(handle) = rigid_body.handle
            {
                (
                    Some((transform.translation, transform.rotation)),
                    Some(handle),
                )
            } else {
                (None, None)
            }
        };

        if let (Some((translation, rotation)), Some(handle)) = (transform_data, handle) {
            let rapier_translation = vector![translation.x, translation.y, translation.z];
            let rapier_rotation = UnitQuaternion::new_normalize(Quaternion::new(
                rotation.w, rotation.i, rotation.j, rotation.k,
            ));

            if let Some(rb) = physics_world_get_rigid_body_mut(
                world.plugin_resource_mut::<PhysicsWorld>(),
                super::types::rigid_body_handle_to_rapier(handle),
            ) {
                if rb.body_type() == RigidBodyType::KinematicPositionBased {
                    rb.set_next_kinematic_translation(rapier_translation);
                    rb.set_next_kinematic_rotation(rapier_rotation);
                } else if rb.body_type() == RigidBodyType::Fixed {
                    let translation_unchanged =
                        (rb.translation() - rapier_translation).norm_squared() < 1.0e-8;
                    let rotation_unchanged = rb.rotation().angle_to(&rapier_rotation) < 1.0e-5;
                    if !translation_unchanged || !rotation_unchanged {
                        rb.set_translation(rapier_translation, true);
                        rb.set_rotation(rapier_rotation, true);
                    }
                }
            }
        }
    }
}

pub fn initialize_physics_bodies_system(world: &mut World) {
    let entities: Vec<_> = world.ecs.worlds[CORE]
        .query_entities(crate::ecs::world::RIGID_BODY)
        .collect();

    for entity in entities {
        let should_initialize = world
            .get::<crate::plugins::physics::components::RigidBodyComponent>(entity)
            .map(|rb| rb.handle.is_none())
            .unwrap_or(false);

        if !should_initialize {
            continue;
        }

        let rigid_body_data = world
            .get::<crate::plugins::physics::components::RigidBodyComponent>(entity)
            .cloned();
        let collider_data = world
            .get::<crate::plugins::physics::components::ColliderComponent>(entity)
            .cloned();

        if let Some(rigid_body_comp) = rigid_body_data {
            let rigid_body = rigid_body_comp.to_rapier_rigid_body();
            let rapier_handle = physics_world_add_rigid_body(
                world.plugin_resource_mut::<PhysicsWorld>(),
                rigid_body,
            );
            let has_listener = world
                .get::<crate::plugins::physics::components::CollisionListener>(entity)
                .is_some();

            if let Some(collider_comp) = collider_data {
                let mut collider = collider_comp.to_rapier_collider();
                if has_listener {
                    collider.set_active_events(
                        ActiveEvents::COLLISION_EVENTS | ActiveEvents::CONTACT_FORCE_EVENTS,
                    );
                }
                physics_world_add_collider(
                    world.plugin_resource_mut::<PhysicsWorld>(),
                    collider,
                    rapier_handle,
                );
            }

            if let Some(rigid_body_mut) =
                world.get_mut::<crate::plugins::physics::components::RigidBodyComponent>(entity)
            {
                rigid_body_mut.handle =
                    Some(super::types::rigid_body_handle_from_rapier(rapier_handle));
            }

            world
                .plugin_resource_mut::<PhysicsWorld>()
                .handle_to_entity
                .insert(rapier_handle, entity);
            world
                .plugin_resource_mut::<PhysicsWorld>()
                .entity_to_handle
                .insert(entity, rapier_handle);

            let transform_data = world
                .get::<crate::ecs::transform::components::LocalTransform>(entity)
                .map(|t| (t.translation, t.rotation));

            if let Some((translation, rotation)) = transform_data
                && let Some(interpolation) =
                    world.get_mut::<crate::plugins::physics::components::PhysicsInterpolation>(
                        entity,
                    )
            {
                interpolation.previous_translation = translation;
                interpolation.previous_rotation = rotation;
                interpolation.current_translation = translation;
                interpolation.current_rotation = rotation;
            }
        }
    }
}

pub fn process_pending_joints_system(world: &mut World) {
    use freecs::dynamic::ResourceHostExt;
    world.resource_scope::<PhysicsWorld, _>(|world, physics| {
        let mut indices_to_remove = Vec::new();
        let mut joints_to_register = Vec::new();

        for (index, pending) in physics.pending_joints.iter().enumerate() {
            let parent_rb = world.get::<crate::plugins::physics::components::RigidBodyComponent>(
                pending.parent_entity,
            );
            let child_rb = world.get::<crate::plugins::physics::components::RigidBodyComponent>(
                pending.child_entity,
            );

            match (parent_rb, child_rb) {
                (Some(parent), Some(child)) => {
                    if let (Some(parent_handle), Some(child_handle)) = (parent.handle, child.handle)
                    {
                        let joint_handle = physics.impulse_joint_set.insert(
                            super::types::rigid_body_handle_to_rapier(parent_handle),
                            super::types::rigid_body_handle_to_rapier(child_handle),
                            pending.joint,
                            pending.collisions_enabled,
                        );
                        joints_to_register.push((
                            joint_handle,
                            super::resources::JointInfo {
                                parent_entity: pending.parent_entity,
                                child_entity: pending.child_entity,
                                joint_type: pending.joint_type,
                                collisions_enabled: pending.collisions_enabled,
                                spring_rest_length: pending.spring_rest_length,
                                spring_stiffness: pending.spring_stiffness,
                                spring_damping: pending.spring_damping,
                            },
                        ));
                        indices_to_remove.push(index);
                    }
                }
                (None, _) | (_, None) => {
                    indices_to_remove.push(index);
                }
            }
        }

        for (handle, info) in joints_to_register {
            physics.joint_registry.insert(handle, info);
        }

        for index in indices_to_remove.into_iter().rev() {
            physics.pending_joints.swap_remove(index);
        }
    });
}

pub fn cleanup_physics_bodies_system(world: &mut World) {
    let mut to_remove = Vec::new();

    for (rapier_handle, _) in world
        .plugin_resource::<PhysicsWorld>()
        .rigid_body_set
        .iter()
    {
        if let Some(&entity) = world
            .plugin_resource::<PhysicsWorld>()
            .handle_to_entity
            .get(&rapier_handle)
        {
            let has_component = world
                .get::<crate::plugins::physics::components::RigidBodyComponent>(entity)
                .is_some();
            if !has_component {
                to_remove.push(rapier_handle);
            }
        }
    }

    for rapier_handle in to_remove {
        physics_world_remove_rigid_body(world.plugin_resource_mut::<PhysicsWorld>(), rapier_handle);
    }
}

pub fn physics_interpolation_system(world: &mut World) {
    let alpha = physics_world_get_interpolation_alpha(world.plugin_resource::<PhysicsWorld>());

    let interpolation_entities: Vec<_> = world.ecs.worlds[CORE]
        .query_entities(crate::ecs::world::PHYSICS_INTERPOLATION)
        .collect();

    for entity in interpolation_entities {
        let interpolation_data = world
            .get::<crate::plugins::physics::components::PhysicsInterpolation>(entity)
            .and_then(|i| {
                if i.enabled {
                    Some(i.interpolate(alpha))
                } else {
                    None
                }
            });

        if let Some((new_translation, new_rotation)) = interpolation_data {
            let should_update = if let Some(transform) =
                world.get::<crate::ecs::transform::components::LocalTransform>(entity)
            {
                !transforms_approximately_equal(
                    &transform.translation,
                    &new_translation,
                    &transform.rotation,
                    &new_rotation,
                )
            } else {
                true
            };

            if should_update
                && let Some(transform) =
                    world.get_mut::<crate::ecs::transform::components::LocalTransform>(entity)
            {
                transform.translation = new_translation;
                transform.rotation = new_rotation;
            }
        }
    }
}

fn sync_collision_listener_system(world: &mut World) {
    let entities: Vec<_> = world.ecs.worlds[CORE]
        .query_entities(crate::ecs::world::COLLISION_LISTENER | crate::ecs::world::COLLIDER)
        .collect();

    for entity in entities {
        let collider_handle = world
            .get::<crate::plugins::physics::components::ColliderComponent>(entity)
            .and_then(|c| c.handle);
        if let Some(handle) = collider_handle
            && let Some(collider) = world
                .plugin_resource_mut::<PhysicsWorld>()
                .collider_set
                .get_mut(super::types::collider_handle_to_rapier(handle))
            && !collider
                .active_events()
                .contains(ActiveEvents::COLLISION_EVENTS)
        {
            collider.set_active_events(
                ActiveEvents::COLLISION_EVENTS | ActiveEvents::CONTACT_FORCE_EVENTS,
            );
        }
    }
}

fn drain_collision_events(world: &mut World) {
    let raw_collisions = world
        .plugin_resource_mut::<PhysicsWorld>()
        .event_collector
        .drain_raw_collisions();
    let raw_forces = world
        .plugin_resource_mut::<PhysicsWorld>()
        .event_collector
        .drain_raw_contact_forces();

    let mut collision_events = Vec::with_capacity(raw_collisions.len());
    for raw in raw_collisions {
        let entity_a = resolve_collider_to_entity(world, raw.collider_a);
        let entity_b = resolve_collider_to_entity(world, raw.collider_b);
        if let (Some(entity_a), Some(entity_b)) = (entity_a, entity_b) {
            let has_listener = world
                .get::<crate::plugins::physics::components::CollisionListener>(entity_a)
                .is_some()
                || world
                    .get::<crate::plugins::physics::components::CollisionListener>(entity_b)
                    .is_some();
            if has_listener {
                collision_events.push(CollisionEvent {
                    entity_a,
                    entity_b,
                    kind: if raw.started {
                        CollisionEventKind::Started
                    } else {
                        CollisionEventKind::Stopped
                    },
                    is_sensor: raw.is_sensor,
                });
            }
        }
    }

    let mut contact_force_events = Vec::with_capacity(raw_forces.len());
    for raw in raw_forces {
        let entity_a = resolve_collider_to_entity(world, raw.collider_a);
        let entity_b = resolve_collider_to_entity(world, raw.collider_b);
        if let (Some(entity_a), Some(entity_b)) = (entity_a, entity_b) {
            let has_listener = world
                .get::<crate::plugins::physics::components::CollisionListener>(entity_a)
                .is_some()
                || world
                    .get::<crate::plugins::physics::components::CollisionListener>(entity_b)
                    .is_some();
            if has_listener {
                contact_force_events.push(ContactForceEvent {
                    entity_a,
                    entity_b,
                    total_force_magnitude: raw.total_force_magnitude,
                });
            }
        }
    }

    physics_world_set_collision_events(
        world.plugin_resource_mut::<PhysicsWorld>(),
        collision_events,
    );
    physics_world_set_contact_force_events(
        world.plugin_resource_mut::<PhysicsWorld>(),
        contact_force_events,
    );
}

fn resolve_collider_to_entity(
    world: &World,
    collider_handle: ColliderHandle,
) -> Option<freecs::Entity> {
    let collider = world
        .plugin_resource::<PhysicsWorld>()
        .collider_set
        .get(collider_handle)?;
    let rb_handle = collider.parent()?;
    world
        .plugin_resource::<PhysicsWorld>()
        .handle_to_entity
        .get(&rb_handle)
        .copied()
}
