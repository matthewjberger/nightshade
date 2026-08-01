use nalgebra_glm::vec3;
use nightshade::ecs::Entity;
use nightshade_render::components::mesh::InstancedMesh;
use nightshade_render::data::bounding_volume::{BoundingVolume, OrientedBoundingBox};
use nightshade_render::data::generational_registry::registry_entry_by_name;
use nightshade_render::resources::mesh_cache::MeshCache;

fn resolve_bounding_volume(mesh_cache: &MeshCache, mesh_name: &str) -> BoundingVolume {
    if let Some(mesh) = registry_entry_by_name(&mesh_cache.registry, mesh_name)
        && let Some(bv) = &mesh.bounding_volume
    {
        return *bv;
    }
    BoundingVolume::from_mesh_type(mesh_name)
}

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

pub fn ensure_bounding_volumes(
    mesh_cache: nightshade::ecs::system_param::Res<MeshCache>,
    storage: &mut nightshade::ecs::Storage,
) {
    let mut pending: Vec<(Entity, BoundingVolume)> = Vec::new();

    for (entity, render_mesh) in storage
        .query_ref::<&nightshade_render::components::mesh::RenderMesh>()
        .without::<BoundingVolume>()
        .iter()
    {
        pending.push((
            entity,
            resolve_bounding_volume(&mesh_cache, &render_mesh.name),
        ));
    }
    for (entity, instanced_mesh) in storage
        .query_ref::<&InstancedMesh>()
        .without::<BoundingVolume>()
        .iter()
    {
        pending.push((
            entity,
            instanced_bounding_volume(&mesh_cache, instanced_mesh),
        ));
    }

    for (entity, bv) in pending {
        storage.set(entity, bv);
    }
}
