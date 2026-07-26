//! How physics joins prefab spawning.
//!
//! The prefab record keeps its own plain `PrefabRigidBody` and `PrefabCollider`;
//! turning those into live bodies and colliders is this plugin's job, registered
//! as a prefab hook when the plugin is composed.

use crate::plugins::physics::components::{ColliderComponent, ColliderShape, RigidBodyComponent};
use crate::plugins::physics::types::{InteractionGroups, LockedAxes, RigidBodyType};

use crate::ecs::transform::components::LocalTransform;
use crate::ecs::world::World;
use crate::plugins::physics::resources::{
    PhysicsWorld, physics_world_add_collider, physics_world_add_rigid_body,
};
use nightshade_ecs::Entity;

use crate::assets::prefab::components::{
    PrefabBodyType, PrefabCollider, PrefabColliderShape, PrefabComponents, PrefabRigidBody,
};

fn apply_prefab_physics(
    world: &mut World,
    entity: Entity,
    prefab_rigid_body: &PrefabRigidBody,
    prefab_collider: Option<&PrefabCollider>,
    local_transform: LocalTransform,
) {
    if world.ecs.resource::<PhysicsWorld>().is_none() {
        return;
    }
    crate::plugins::physics::schema::physics_component_world(world);
    let mut rigid_body_component = prefab_rigid_body_to_component(prefab_rigid_body);
    rigid_body_component.translation = local_transform.translation.into();
    let quat = local_transform.rotation;
    rigid_body_component.rotation = [quat.i, quat.j, quat.k, quat.w];

    let rapier_rigid_body = rigid_body_component.to_rapier_rigid_body();
    let rapier_handle = physics_world_add_rigid_body(
        world.plugin_resource_mut::<PhysicsWorld>(),
        rapier_rigid_body,
    );
    rigid_body_component.handle =
        Some(crate::plugins::physics::types::rigid_body_handle_from_rapier(rapier_handle));

    if let Some(prefab_collider) = prefab_collider {
        let mut collider_component = prefab_collider_to_component(prefab_collider);
        let rapier_collider = collider_component.to_rapier_collider();
        let rapier_collider_handle = physics_world_add_collider(
            world.plugin_resource_mut::<PhysicsWorld>(),
            rapier_collider,
            rapier_handle,
        );
        collider_component.handle = Some(
            crate::plugins::physics::types::collider_handle_from_rapier(rapier_collider_handle),
        );
        world.set(entity, collider_component);
    }

    world.set(entity, rigid_body_component);
}

pub fn prefab_body_type_to_rigid_body_type(body_type: PrefabBodyType) -> RigidBodyType {
    match body_type {
        PrefabBodyType::Dynamic => RigidBodyType::Dynamic,
        PrefabBodyType::Kinematic => RigidBodyType::KinematicPositionBased,
        PrefabBodyType::Static => RigidBodyType::Fixed,
    }
}

pub fn prefab_rigid_body_to_component(prefab: &PrefabRigidBody) -> RigidBodyComponent {
    RigidBodyComponent {
        handle: None,
        body_type: prefab_body_type_to_rigid_body_type(prefab.body_type),
        translation: [0.0, 0.0, 0.0],
        rotation: [0.0, 0.0, 0.0, 1.0],
        linvel: [0.0, 0.0, 0.0],
        angvel: [0.0, 0.0, 0.0],
        mass: prefab.mass,
        linear_damping: 0.0,
        angular_damping: 0.0,
        gravity_scale: 1.0,
        locked_axes: LockedAxes::empty(),
        ccd_enabled: false,
    }
}

pub fn prefab_collider_to_component(prefab: &PrefabCollider) -> ColliderComponent {
    ColliderComponent {
        handle: None,
        shape: collider_shape_from_prefab(&prefab.shape),
        friction: prefab.friction,
        restitution: prefab.restitution,
        density: prefab.density,
        is_sensor: prefab.is_sensor,
        collision_groups: InteractionGroups::all(),
        solver_groups: InteractionGroups::all(),
    }
}

/// Spawns this plugin's slice of a prefab node.
pub fn spawn_from_prefab(
    world: &mut World,
    entity: Entity,
    components: &PrefabComponents,
    local_transform: LocalTransform,
) {
    if let Some(rigid_body) = &components.rigid_body {
        apply_prefab_physics(
            world,
            entity,
            rigid_body,
            components.collider.as_ref(),
            local_transform,
        );
    }
}

/// Appends this plugin's prefab hook. Called from the plugin's `build`.
pub fn register_prefab_hooks(world: &mut World) {
    world
        .res_mut::<crate::assets::prefab::hooks::PrefabCapabilityHooks>()
        .entity_spawn
        .push(spawn_from_prefab);
}

/// The record's shape vocabulary mapped onto the plugin's.
fn collider_shape_from_prefab(shape: &PrefabColliderShape) -> ColliderShape {
    match shape {
        PrefabColliderShape::Ball { radius } => ColliderShape::Ball { radius: *radius },
        PrefabColliderShape::Cuboid { hx, hy, hz } => ColliderShape::Cuboid {
            hx: *hx,
            hy: *hy,
            hz: *hz,
        },
        PrefabColliderShape::Capsule {
            half_height,
            radius,
        } => ColliderShape::Capsule {
            half_height: *half_height,
            radius: *radius,
        },
        PrefabColliderShape::Cylinder {
            half_height,
            radius,
        } => ColliderShape::Cylinder {
            half_height: *half_height,
            radius: *radius,
        },
        PrefabColliderShape::Cone {
            half_height,
            radius,
        } => ColliderShape::Cone {
            half_height: *half_height,
            radius: *radius,
        },
        PrefabColliderShape::ConvexMesh { vertices } => ColliderShape::ConvexMesh {
            vertices: vertices.clone(),
        },
        PrefabColliderShape::TriMesh { vertices, indices } => ColliderShape::TriMesh {
            vertices: vertices.clone(),
            indices: indices.clone(),
        },
        PrefabColliderShape::HeightField {
            nrows,
            ncols,
            heights,
            scale,
        } => ColliderShape::HeightField {
            nrows: *nrows,
            ncols: *ncols,
            heights: heights.clone(),
            scale: *scale,
        },
    }
}
