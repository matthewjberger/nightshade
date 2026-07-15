use crate::ecs::mesh::components::InstancedMesh;
use crate::ecs::world::{Entity, World};
use crate::render::bounding_volume::{BoundingVolume, OrientedBoundingBox};
use crate::render::generational_registry::registry_entry_by_name;
use crate::render::mesh_cache::MeshCache;
use nalgebra_glm::vec3;

fn resolve_bounding_volume(mesh_cache: &MeshCache, mesh_name: &str) -> BoundingVolume {
    if let Some(mesh) = registry_entry_by_name(&mesh_cache.registry, mesh_name)
        && let Some(bv) = &mesh.bounding_volume
    {
        return *bv;
    }
    BoundingVolume::from_mesh_type(mesh_name)
}

/// A bounding volume that encloses every instance of an instanced mesh, in the
/// entity's local space. Culling transforms it by the entity's global transform,
/// so it must cover the instances themselves rather than the base mesh sitting at
/// the entity origin — otherwise a batch whose instances are spread far from the
/// origin (e.g. a streamed scatter chunk) culls on the origin's visibility and
/// pops in and out. Falls back to the base mesh volume when there are no instances.
fn instanced_bounding_volume(mesh_cache: &MeshCache, instanced: &InstancedMesh) -> BoundingVolume {
    let base = resolve_bounding_volume(mesh_cache, &instanced.mesh_name);
    if instanced.instances.is_empty() {
        return base;
    }
    let mut min = vec3(f32::MAX, f32::MAX, f32::MAX);
    let mut max = vec3(f32::MIN, f32::MIN, f32::MIN);
    for instance in &instanced.instances {
        let obb = base.obb.transform(&instance.as_matrix());
        for corner in obb.get_corners() {
            min = min.inf(&corner);
            max = max.sup(&corner);
        }
    }
    BoundingVolume::new(
        OrientedBoundingBox::from_aabb(min, max),
        (max - min).norm() * 0.5,
    )
}

/// Adds [`BoundingVolume`] components to mesh entities that don't have one yet.
/// Picking, frustum culling, and depth-aware camera systems rely on this data,
/// so apps that load meshes outside the gizmo / scene-spawn paths should call
/// this each frame (or after batched spawns).
pub fn ensure_bounding_volumes(world: &mut World) {
    let mut pending: Vec<(Entity, BoundingVolume)> = Vec::new();

    let mesh_cache = &world
        .expect_resource::<crate::ecs::asset_state::AssetState>()
        .mesh_cache;
    for (entity, render_mesh) in world
        .query_ref::<&crate::ecs::mesh::components::RenderMesh>()
        .without::<BoundingVolume>()
        .iter()
    {
        pending.push((
            entity,
            resolve_bounding_volume(mesh_cache, &render_mesh.name),
        ));
    }
    for (entity, instanced_mesh) in world
        .query_ref::<&InstancedMesh>()
        .without::<BoundingVolume>()
        .iter()
    {
        pending.push((
            entity,
            instanced_bounding_volume(mesh_cache, instanced_mesh),
        ));
    }

    for (entity, bv) in pending {
        world.set(entity, bv);
    }
}
