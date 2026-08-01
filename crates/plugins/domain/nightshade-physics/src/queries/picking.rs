use nightshade::ecs::QueryRef;
use nightshade_picking::{PickingOptions, PickingRay, PickingResult};

pub type VisibilityRows<'world> =
    QueryRef<'world, (&'world nightshade_scene::contract::primitives::Visibility,)>;

pub fn pick_entities_trimesh(
    picking_world: &crate::resources::PickingWorld,
    rows: VisibilityRows<'_>,
    ray: PickingRay,
    options: PickingOptions,
) -> Vec<PickingResult> {
    use rapier3d::prelude::*;

    let rapier_ray = rapier3d::parry::query::Ray::new(
        point![ray.origin.x, ray.origin.y, ray.origin.z],
        vector![ray.direction.x, ray.direction.y, ray.direction.z],
    );

    let hidden: std::collections::HashSet<nightshade::ecs::Entity> = if options.ignore_invisible {
        rows.iter()
            .filter(|(_, (visible,))| !visible.visible)
            .map(|(entity, _)| entity)
            .collect()
    } else {
        std::collections::HashSet::new()
    };

    let mut results = Vec::new();
    for (handle, collider) in picking_world.collider_set.iter() {
        let Some(entity) = picking_world.get_entity(handle) else {
            continue;
        };
        if hidden.contains(&entity) {
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

pub fn pick_closest_entity_trimesh(
    picking_world: &crate::resources::PickingWorld,
    rows: VisibilityRows<'_>,
    ray: PickingRay,
) -> Option<PickingResult> {
    pick_entities_trimesh(picking_world, rows, ray, PickingOptions::default())
        .into_iter()
        .next()
}
