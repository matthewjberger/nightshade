use nightshade::ecs::Entity;
use rapier3d::prelude::*;

use crate::resources::PhysicsWorld;
use crate::resources::events::{CollisionEvent, ContactForceEvent};

pub fn physics_world_collision_events(physics: &PhysicsWorld) -> &[CollisionEvent] {
    &physics.collision_events
}

pub fn physics_world_contact_force_events(physics: &PhysicsWorld) -> &[ContactForceEvent] {
    &physics.contact_force_events
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

pub fn physics_world_cast_ray(
    physics: &PhysicsWorld,
    origin: nalgebra_glm::Vec3,
    direction: nalgebra_glm::Vec3,
    max_distance: f32,
    exclude: Option<Entity>,
) -> Option<crate::contract::types::RaycastHit> {
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
    Some(crate::contract::types::RaycastHit {
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

pub fn physics_world_capsule_contacts(
    physics: &PhysicsWorld,
    center: nalgebra_glm::Vec3,
    half_height: f32,
    radius: f32,
    exclude: Option<Entity>,
) -> Vec<crate::contract::types::ShapeContact> {
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
        contacts.push(crate::contract::types::ShapeContact {
            entity,
            point: nalgebra_glm::vec3(contact.point1.x, contact.point1.y, contact.point1.z),
            normal: nalgebra_glm::vec3(-contact.normal1.x, -contact.normal1.y, -contact.normal1.z),
            depth: -contact.dist,
        });
    }
    contacts
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

pub fn physics_world_angular_velocity(
    physics: &PhysicsWorld,
    entity: Entity,
) -> Option<nalgebra_glm::Vec3> {
    let handle = physics.entity_to_handle.get(&entity)?;
    let rb = physics.rigid_body_set.get(*handle)?;
    let velocity = rb.angvel();
    Some(nalgebra_glm::vec3(velocity.x, velocity.y, velocity.z))
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
