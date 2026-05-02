use crate::ecs::bounding_volume::components::BoundingVolume;
use crate::ecs::generational_registry::registry_entry_by_name;
use crate::ecs::world::{BOUNDING_VOLUME, Entity, INSTANCED_MESH, RENDER_MESH, World};

fn resolve_bounding_volume(world: &World, mesh_name: &str) -> BoundingVolume {
    if let Some(mesh) = registry_entry_by_name(&world.resources.mesh_cache.registry, mesh_name)
        && let Some(bv) = &mesh.bounding_volume
    {
        return *bv;
    }
    BoundingVolume::from_mesh_type(mesh_name)
}

/// Adds [`BoundingVolume`] components to mesh entities that don't have one yet.
/// Picking, frustum culling, and depth-aware camera systems rely on this data,
/// so apps that load meshes outside the gizmo / scene-spawn paths should call
/// this each frame (or after batched spawns).
pub fn ensure_bounding_volumes(world: &mut World) {
    let render_mesh_entities: Vec<Entity> = world
        .core
        .query_entities(RENDER_MESH)
        .filter(|entity| !world.core.entity_has_components(*entity, BOUNDING_VOLUME))
        .collect();

    for entity in render_mesh_entities {
        let mesh_name = world
            .core
            .get_render_mesh(entity)
            .map(|rm| rm.name.clone())
            .unwrap_or_default();

        let bounding_volume = resolve_bounding_volume(world, &mesh_name);
        world.core.add_components(entity, BOUNDING_VOLUME);
        world.core.set_bounding_volume(entity, bounding_volume);
    }

    let instanced_mesh_entities: Vec<Entity> = world
        .core
        .query_entities(INSTANCED_MESH)
        .filter(|entity| !world.core.entity_has_components(*entity, BOUNDING_VOLUME))
        .collect();

    for entity in instanced_mesh_entities {
        let mesh_name = world
            .core
            .get_instanced_mesh(entity)
            .map(|im| im.mesh_name.clone())
            .unwrap_or_default();

        let bounding_volume = resolve_bounding_volume(world, &mesh_name);
        world.core.add_components(entity, BOUNDING_VOLUME);
        world.core.set_bounding_volume(entity, bounding_volume);
    }
}
