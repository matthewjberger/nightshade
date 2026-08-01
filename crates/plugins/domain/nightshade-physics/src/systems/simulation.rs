use crate::commands::physics_world::{
    physics_world_add_collider, physics_world_add_rigid_body, physics_world_clear_events,
    physics_world_get_rigid_body_mut, physics_world_remove_rigid_body,
    physics_world_set_collision_events, physics_world_set_contact_force_events, physics_world_step,
};
use crate::components::JustInitialized;
use crate::contract::components::{
    ColliderComponent, CollisionListener, PhysicsInterpolation, RigidBodyComponent,
};
use crate::resources::PhysicsWorld;
use crate::resources::events::{CollisionEvent, CollisionEventKind, ContactForceEvent};
use nalgebra_glm;
use nightshade::World;
use nightshade::ecs::system_param::{Query, ResMut, run_system};
use nightshade_scene::contract::LocalTransform;
use rapier3d::na::{Quaternion, UnitQuaternion};
use rapier3d::prelude::*;
use tracing;

use crate::prelude::*;
pub fn run_physics_systems(world: &mut World) {
    physics_world_clear_events(world.res_mut::<PhysicsWorld>());

    run_system(world, initialize_physics_bodies_system);
    run_system(world, seed_physics_interpolation_system);
    run_system(world, sync_collision_listener_system);
    run_system(world, process_pending_joints_system);

    if !world.res::<PhysicsWorld>().enabled {
        return;
    }

    let fixed_timestep = world.res::<PhysicsWorld>().fixed_timestep;
    let max_substeps = world.res::<PhysicsWorld>().max_substeps;
    let max_frame_time = max_substeps as f32 * fixed_timestep;
    let frame_time = world
        .res::<nightshade_platform::contract::time::Time>()
        .delta_time
        .min(max_frame_time);

    world.res_mut::<PhysicsWorld>().accumulator += frame_time;

    let mut substeps_performed = 0;

    while world.res::<PhysicsWorld>().accumulator >= fixed_timestep
        && substeps_performed < max_substeps
    {
        run_system(world, advance_interpolation_previous_and_restore_transform);

        run_system(world, sync_transforms_to_physics_system);

        run_system(
            world,
            crate::systems::character::character_controller_input_system,
        );
        run_system(
            world,
            crate::systems::character::character_controller_system,
        );

        physics_world_step(world.res_mut::<PhysicsWorld>());

        run_system(world, sync_interpolation_current_from_physics);

        world.res_mut::<PhysicsWorld>().accumulator -= fixed_timestep;
        substeps_performed += 1;
    }

    run_system(world, drain_collision_events);

    tracing::trace!(substeps = substeps_performed);

    let interpolation_alpha =
        (world.res::<PhysicsWorld>().accumulator / fixed_timestep).clamp(0.0, 1.0);
    world.res_mut::<PhysicsWorld>().interpolation_alpha = interpolation_alpha;

    run_system(world, sync_transforms_from_physics_system);
    run_system(world, physics_interpolation_system);

    run_system(world, cleanup_physics_bodies_system);
    run_system(world, crate::systems::debug::physics_debug_draw_system);
}

fn advance_interpolation_previous_and_restore_transform(
    physics: Res<PhysicsWorld>,
    mut commands: Commands,
    bodies: Query<(
        &mut crate::contract::components::PhysicsInterpolation,
        &nightshade_scene::contract::LocalTransform,
    )>,
) {
    bodies.for_each(|entity, (interpolation, transform)| {
        let body_type = physics
            .entity_to_handle
            .get(&entity)
            .and_then(|handle| physics.rigid_body_set.get(*handle))
            .map(|rigid_body| rigid_body.body_type());

        let kinematic = matches!(
            body_type,
            Some(RigidBodyType::KinematicPositionBased | RigidBodyType::KinematicVelocityBased)
        );

        interpolation.update_previous();
        if kinematic || !interpolation.enabled {
            return;
        }

        if interpolation.current_translation == transform.translation
            && interpolation.current_rotation == transform.rotation
        {
            return;
        }
        let mut restored = *transform;
        restored.translation = interpolation.current_translation;
        restored.rotation = interpolation.current_rotation;
        commands.set(entity, restored);
    });
}

fn sync_interpolation_current_from_physics(
    physics: Res<PhysicsWorld>,
    mut bodies: Query<(&mut crate::contract::components::PhysicsInterpolation,)>,
) {
    for handle in physics.island_manager.active_bodies() {
        let Some(&entity) = physics.handle_to_entity.get(handle) else {
            continue;
        };
        let Some(rigid_body) = physics.rigid_body_set.get(*handle) else {
            continue;
        };
        let translation = rigid_body.translation();
        let rotation = rigid_body.rotation();
        let new_translation = nalgebra_glm::vec3(translation.x, translation.y, translation.z);
        let new_rotation = nalgebra_glm::quat(rotation.i, rotation.j, rotation.k, rotation.w);
        bodies.for_entity(entity, |(interpolation,)| {
            interpolation.update_current(new_translation, new_rotation);
        });
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

pub fn sync_transforms_from_physics_system(
    physics: Res<PhysicsWorld>,
    mut commands: Commands,
    mut transforms: Query<(&nightshade_scene::contract::LocalTransform,)>,
) {
    for handle in physics.island_manager.active_bodies() {
        let Some(&entity) = physics.handle_to_entity.get(handle) else {
            continue;
        };
        let Some(rigid_body) = physics.rigid_body_set.get(*handle) else {
            continue;
        };

        let translation = rigid_body.translation();
        let rotation = rigid_body.rotation();
        let new_translation = nalgebra_glm::vec3(translation.x, translation.y, translation.z);
        let new_rotation = nalgebra_glm::quat(rotation.i, rotation.j, rotation.k, rotation.w);

        let mut moved = None;
        transforms.for_entity(entity, |(transform,)| {
            if transforms_approximately_equal(
                &transform.translation,
                &new_translation,
                &transform.rotation,
                &new_rotation,
            ) {
                return;
            }
            let mut updated = *transform;
            updated.translation = new_translation;
            updated.rotation = new_rotation;
            moved = Some(updated);
        });

        if let Some(updated) = moved {
            commands.set(entity, updated);
        }
    }
}

pub fn sync_transforms_to_physics_system(
    mut physics: ResMut<PhysicsWorld>,
    bodies: Query<(
        &crate::contract::components::RigidBodyComponent,
        &nightshade_scene::contract::LocalTransform,
    )>,
) {
    let mut pending = Vec::new();
    bodies.for_each(|_entity, (rigid_body, transform)| {
        if let Some(handle) = rigid_body.handle {
            pending.push((handle, transform.translation, transform.rotation));
        }
    });

    for (handle, translation, rotation) in pending {
        let rapier_translation = vector![translation.x, translation.y, translation.z];
        let rapier_rotation = UnitQuaternion::new_normalize(Quaternion::new(
            rotation.w, rotation.i, rotation.j, rotation.k,
        ));

        let Some(rb) = physics_world_get_rigid_body_mut(
            &mut physics,
            crate::data::rapier_convert::rigid_body_handle_to_rapier(handle),
        ) else {
            continue;
        };

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

pub fn initialize_physics_bodies_system(
    mut physics: ResMut<PhysicsWorld>,
    mut commands: Commands,
    bodies: Query<(
        &RigidBodyComponent,
        Option<&ColliderComponent>,
        Option<&CollisionListener>,
    )>,
) {
    bodies.for_each(|entity, (rigid_body, collider, listener)| {
        if rigid_body.handle.is_some() {
            return;
        }

        let rapier_handle = physics_world_add_rigid_body(
            &mut physics,
            crate::data::rapier_convert::rigid_body_to_rapier(rigid_body),
        );

        if let Some(collider_component) = collider {
            let mut collider = crate::data::rapier_convert::collider_to_rapier(collider_component);
            if listener.is_some() {
                collider.set_active_events(
                    ActiveEvents::COLLISION_EVENTS | ActiveEvents::CONTACT_FORCE_EVENTS,
                );
            }
            physics_world_add_collider(&mut physics, collider, rapier_handle);
        }

        let mut initialized = rigid_body.clone();
        initialized.handle = Some(crate::data::rapier_convert::rigid_body_handle_from_rapier(
            rapier_handle,
        ));
        commands.set(entity, initialized);
        commands.add_tag_type::<JustInitialized>(entity);

        physics.handle_to_entity.insert(rapier_handle, entity);
        physics.entity_to_handle.insert(entity, rapier_handle);
    });
}

pub fn seed_physics_interpolation_system(
    mut commands: Commands,
    seeded: Query<(&LocalTransform, &mut PhysicsInterpolation), WithTag<JustInitialized>>,
) {
    seeded.for_each(|entity, (transform, interpolation)| {
        interpolation.previous_translation = transform.translation;
        interpolation.previous_rotation = transform.rotation;
        interpolation.current_translation = transform.translation;
        interpolation.current_rotation = transform.rotation;
        commands.remove_tag_type::<JustInitialized>(entity);
    });
}

pub fn process_pending_joints_system(
    mut physics: ResMut<PhysicsWorld>,
    mut bodies: Query<(&crate::contract::components::RigidBodyComponent,)>,
) {
    let mut resolved = Vec::new();
    let mut spent = Vec::new();

    for (index, pending) in physics.pending_joints.iter().enumerate() {
        let mut parent_handle = None;
        let mut child_handle = None;
        bodies.for_entity(pending.parent_entity, |(rigid_body,)| {
            parent_handle = Some(rigid_body.handle);
        });
        bodies.for_entity(pending.child_entity, |(rigid_body,)| {
            child_handle = Some(rigid_body.handle);
        });

        match (parent_handle, child_handle) {
            (None, _) | (_, None) => spent.push(index),
            (Some(parent), Some(child)) => {
                if let (Some(parent), Some(child)) = (parent, child) {
                    resolved.push((index, parent, child));
                    spent.push(index);
                }
            }
        }
    }

    for (index, parent, child) in resolved {
        let pending = &physics.pending_joints[index];
        let joint = pending.joint;
        let collisions_enabled = pending.collisions_enabled;
        let info = crate::resources::JointInfo {
            parent_entity: pending.parent_entity,
            child_entity: pending.child_entity,
            joint_type: pending.joint_type,
            collisions_enabled,
            spring_rest_length: pending.spring_rest_length,
            spring_stiffness: pending.spring_stiffness,
            spring_damping: pending.spring_damping,
        };
        let joint_handle = physics.impulse_joint_set.insert(
            crate::data::rapier_convert::rigid_body_handle_to_rapier(parent),
            crate::data::rapier_convert::rigid_body_handle_to_rapier(child),
            joint,
            collisions_enabled,
        );
        physics.joint_registry.insert(joint_handle, info);
    }

    for index in spent.into_iter().rev() {
        physics.pending_joints.swap_remove(index);
    }
}

pub fn cleanup_physics_bodies_system(
    mut physics: ResMut<PhysicsWorld>,
    bodies: Query<(&crate::contract::components::RigidBodyComponent,)>,
) {
    let mut live: std::collections::HashSet<nightshade::ecs::Entity> =
        std::collections::HashSet::new();
    bodies.for_each(|entity, _| {
        live.insert(entity);
    });

    let stale: Vec<_> = physics
        .rigid_body_set
        .iter()
        .filter_map(|(handle, _)| {
            physics
                .handle_to_entity
                .get(&handle)
                .filter(|entity| !live.contains(entity))
                .map(|_| handle)
        })
        .collect();

    for handle in stale {
        physics_world_remove_rigid_body(&mut physics, handle);
    }
}

pub fn physics_interpolation_system(
    physics: Res<PhysicsWorld>,
    mut commands: Commands,
    bodies: Query<(&PhysicsInterpolation, &LocalTransform)>,
) {
    let alpha = physics_world_get_interpolation_alpha(&physics);

    bodies.for_each(|entity, (interpolation, transform)| {
        if !interpolation.enabled {
            return;
        }
        let (translation, rotation) = interpolation.interpolate(alpha);
        if transforms_approximately_equal(
            &transform.translation,
            &translation,
            &transform.rotation,
            &rotation,
        ) {
            return;
        }
        let mut moved = *transform;
        moved.translation = translation;
        moved.rotation = rotation;
        commands.set(entity, moved);
    });
}

fn sync_collision_listener_system(
    mut physics: ResMut<PhysicsWorld>,
    listeners: Query<
        (&crate::contract::components::ColliderComponent,),
        nightshade::ecs::system_param::With<crate::contract::components::CollisionListener>,
    >,
) {
    let mut handles = Vec::new();
    listeners.for_each(|_entity, (collider,)| {
        if let Some(handle) = collider.handle {
            handles.push(handle);
        }
    });

    for handle in handles {
        if let Some(collider) =
            physics
                .collider_set
                .get_mut(crate::data::rapier_convert::collider_handle_to_rapier(
                    handle,
                ))
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

fn drain_collision_events(
    mut physics: ResMut<PhysicsWorld>,
    mut listeners: Query<(&crate::contract::components::CollisionListener,)>,
) {
    let raw_collisions = physics.event_collector.drain_raw_collisions();
    let raw_forces = physics.event_collector.drain_raw_contact_forces();

    let mut collision_events = Vec::with_capacity(raw_collisions.len());
    for raw in raw_collisions {
        let entity_a = resolve_collider_to_entity(&physics, raw.collider_a);
        let entity_b = resolve_collider_to_entity(&physics, raw.collider_b);
        if let (Some(entity_a), Some(entity_b)) = (entity_a, entity_b)
            && either_listens(&mut listeners, entity_a, entity_b)
        {
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

    let mut contact_force_events = Vec::with_capacity(raw_forces.len());
    for raw in raw_forces {
        let entity_a = resolve_collider_to_entity(&physics, raw.collider_a);
        let entity_b = resolve_collider_to_entity(&physics, raw.collider_b);
        if let (Some(entity_a), Some(entity_b)) = (entity_a, entity_b)
            && either_listens(&mut listeners, entity_a, entity_b)
        {
            contact_force_events.push(ContactForceEvent {
                entity_a,
                entity_b,
                total_force_magnitude: raw.total_force_magnitude,
            });
        }
    }

    physics_world_set_collision_events(&mut physics, collision_events);
    physics_world_set_contact_force_events(&mut physics, contact_force_events);
}

fn either_listens(
    listeners: &mut Query<(&crate::contract::components::CollisionListener,)>,
    entity_a: nightshade::ecs::Entity,
    entity_b: nightshade::ecs::Entity,
) -> bool {
    let mut listens = false;
    listeners.for_entity(entity_a, |_| listens = true);
    if !listens {
        listeners.for_entity(entity_b, |_| listens = true);
    }
    listens
}

fn resolve_collider_to_entity(
    physics: &PhysicsWorld,
    collider_handle: ColliderHandle,
) -> Option<nightshade::ecs::Entity> {
    let collider = physics.collider_set.get(collider_handle)?;
    let rigid_body_handle = collider.parent()?;
    physics.handle_to_entity.get(&rigid_body_handle).copied()
}
