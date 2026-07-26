//! Ray casts against the picking collider set. The rapier ray and the collider
//! traversal live here; the picking domain keeps only the screen-space math.

use crate::ecs::picking::queries::{PickingOptions, PickingRay, PickingResult};
use crate::ecs::world::{Vec2, World};

pub fn pick_entities_trimesh(
    world: &World,
    screen_pos: Vec2,
    options: PickingOptions,
) -> Vec<PickingResult> {
    use rapier3d::prelude::*;

    let ray = match PickingRay::from_screen_position(world, screen_pos) {
        Some(ray) => ray,
        None => return Vec::new(),
    };

    let rapier_ray = rapier3d::parry::query::Ray::new(
        point![ray.origin.x, ray.origin.y, ray.origin.z],
        vector![ray.direction.x, ray.direction.y, ray.direction.z],
    );

    let mut results = Vec::new();
    let picking_world =
        world.plugin_resource::<crate::plugins::physics::picking::resources::PickingWorld>();

    for (handle, collider) in picking_world.collider_set.iter() {
        let entity = match picking_world.get_entity(handle) {
            Some(e) => e,
            None => continue,
        };

        if options.ignore_invisible
            && let Some(visible) = world.get::<crate::ecs::primitives::Visibility>(entity)
            && !visible.visible
        {
            continue;
        }

        if let Some(toi) =
            collider
                .shape()
                .cast_ray(collider.position(), &rapier_ray, options.max_distance, true)
        {
            let world_position = ray.origin + ray.direction * toi;
            results.push(PickingResult {
                entity,
                distance: toi,
                world_position,
            });
        }
    }

    results.sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap());
    results
}

pub fn pick_closest_entity_trimesh(world: &World, screen_pos: Vec2) -> Option<PickingResult> {
    pick_entities_trimesh(world, screen_pos, PickingOptions::default())
        .into_iter()
        .next()
}
