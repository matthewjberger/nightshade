use crate::ecs::world::World;

pub fn update_instanced_mesh_caches_system(world: &mut World) {
    let _span = tracing::info_span!("instanced_meshes").entered();

    let mut transform_dirty = std::mem::take(
        &mut world
            .res_mut::<crate::ecs::world::scratch::Scratch>()
            .transform_dirty,
    );
    let mut local_matrices_dirty = std::mem::take(
        &mut world
            .res_mut::<crate::ecs::world::scratch::Scratch>()
            .local_matrices_dirty,
    );
    let mut seen = std::mem::take(
        &mut world
            .res_mut::<crate::ecs::world::scratch::Scratch>()
            .instanced_seen,
    );
    transform_dirty.clear();
    local_matrices_dirty.clear();
    seen.clear();
    let mut needs_compute_dispatch = false;

    let mut matrix_entries = world
        .res_mut::<crate::ecs::mesh::components::InstancedMeshMatrices>()
        .take_entries();

    world
        .query::<(
            &mut crate::ecs::mesh::components::InstancedMesh,
            Option<&crate::ecs::transform::components::GlobalTransform>,
        )>()
        .for_each(|entity, (instanced_mesh, global_transform)| {
            let parent_transform = global_transform
                .map(|global| global.0)
                .unwrap_or_else(nalgebra_glm::Mat4::identity);
            let was_dirty = instanced_mesh.dirty;
            let had_gpu_dirty = instanced_mesh.local_matrices_gpu_dirty();
            let parent_only_changed = matrix_entries.entry(entity).or_default().update(
                &instanced_mesh.instances,
                instanced_mesh.dirty,
                &parent_transform,
            );
            instanced_mesh.dirty = false;
            let custom_data_dirty = !instanced_mesh.take_dirty_custom_data_indices().is_empty();
            seen.insert(entity);
            if was_dirty || custom_data_dirty {
                transform_dirty.push(entity);
            }
            if had_gpu_dirty {
                local_matrices_dirty.push(entity);
                instanced_mesh.clear_local_matrices_gpu_dirty();
            }
            if parent_only_changed {
                needs_compute_dispatch = true;
            }
        });

    matrix_entries.retain(|entity, _| seen.contains(entity));
    world
        .res_mut::<crate::ecs::mesh::components::InstancedMeshMatrices>()
        .restore_entries(matrix_entries);

    for entity in transform_dirty.drain(..) {
        world
            .res_mut::<crate::render::mesh_state::MeshRenderState>()
            .mark_transform_dirty(entity);
    }
    for entity in local_matrices_dirty.drain(..) {
        world
            .res_mut::<crate::render::mesh_state::MeshRenderState>()
            .mark_instanced_local_matrices_dirty(entity);
    }
    if needs_compute_dispatch {
        world
            .res_mut::<crate::render::mesh_state::MeshRenderState>()
            .mark_instanced_compute_dispatch_needed();
    }

    world
        .res_mut::<crate::ecs::world::scratch::Scratch>()
        .transform_dirty = transform_dirty;
    world
        .res_mut::<crate::ecs::world::scratch::Scratch>()
        .local_matrices_dirty = local_matrices_dirty;
    world
        .res_mut::<crate::ecs::world::scratch::Scratch>()
        .instanced_seen = seen;
}
