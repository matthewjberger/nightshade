use crate::ecs::bounding_volume::components::BoundingVolume;
use crate::ecs::generational_registry::registry_entry_by_name;
use crate::ecs::prefab::resources::MeshCache;
use crate::ecs::world::{BOUNDING_VOLUME, Entity, INSTANCED_MESH, RENDER_MESH, World};

fn resolve_bounding_volume(mesh_cache: &MeshCache, mesh_name: &str) -> BoundingVolume {
    if let Some(mesh) = registry_entry_by_name(&mesh_cache.registry, mesh_name)
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
    let mut pending: Vec<(Entity, BoundingVolume)> = Vec::new();

    let mesh_cache = &world.resources.mesh_cache;
    world
        .core
        .query()
        .with(RENDER_MESH)
        .without(BOUNDING_VOLUME)
        .iter(|entity, table, idx| {
            let mesh_name = &table.render_mesh[idx].name;
            pending.push((entity, resolve_bounding_volume(mesh_cache, mesh_name)));
        });
    world
        .core
        .query()
        .with(INSTANCED_MESH)
        .without(BOUNDING_VOLUME)
        .iter(|entity, table, idx| {
            let mesh_name = &table.instanced_mesh[idx].mesh_name;
            pending.push((entity, resolve_bounding_volume(mesh_cache, mesh_name)));
        });

    for (entity, bv) in pending {
        world.core.add_components(entity, BOUNDING_VOLUME);
        world.core.set_bounding_volume(entity, bv);
    }
}
