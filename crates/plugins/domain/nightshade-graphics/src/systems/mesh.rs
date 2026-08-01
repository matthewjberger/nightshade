use nightshade::prelude::*;

pub fn update_instanced_mesh_caches_system(
    mut scratch: ResMut<nightshade_render::resources::scratch::Scratch>,
    mut matrices: ResMut<nightshade_render::components::mesh::InstancedMeshMatrices>,
    mut mesh_state: ResMut<nightshade_render::resources::mesh_state::MeshRenderState>,
    storage: &mut nightshade::ecs::Storage,
) {
    let mut transform_dirty = std::mem::take(&mut scratch.transform_dirty);
    let mut local_matrices_dirty = std::mem::take(&mut scratch.local_matrices_dirty);
    let mut seen = std::mem::take(&mut scratch.instanced_seen);
    transform_dirty.clear();
    local_matrices_dirty.clear();
    seen.clear();
    let mut needs_compute_dispatch = false;
    let mut settled: Vec<Entity> = Vec::new();

    let mut matrix_entries = matrices.take_entries();

    for (entity, (instanced_mesh, global_transform)) in storage
        .query_ref::<(
            &nightshade_render::components::mesh::InstancedMesh,
            Option<&nightshade_scene::contract::GlobalTransform>,
        )>()
        .iter()
    {
        let parent_transform = global_transform
            .map(|global| global.0)
            .unwrap_or_else(nalgebra_glm::Mat4::identity);
        let was_dirty = instanced_mesh.dirty;
        let had_gpu_dirty = instanced_mesh.local_matrices_gpu_dirty();
        let custom_data_dirty = instanced_mesh.has_dirty_custom_data();
        let parent_only_changed = matrix_entries.entry(entity).or_default().update(
            &instanced_mesh.instances,
            was_dirty,
            &parent_transform,
        );
        seen.insert(entity);
        if was_dirty || custom_data_dirty {
            transform_dirty.push(entity);
        }
        if had_gpu_dirty {
            local_matrices_dirty.push(entity);
        }
        if was_dirty || had_gpu_dirty || custom_data_dirty {
            settled.push(entity);
        }
        if parent_only_changed {
            needs_compute_dispatch = true;
        }
    }

    for entity in settled {
        if let Some(instanced_mesh) =
            storage.get_mut::<nightshade_render::components::mesh::InstancedMesh>(entity)
        {
            instanced_mesh.dirty = false;
            instanced_mesh.take_dirty_custom_data_indices();
            instanced_mesh.clear_local_matrices_gpu_dirty();
        }
    }

    matrix_entries.retain(|entity, _| seen.contains(entity));
    matrices.restore_entries(matrix_entries);

    for entity in transform_dirty.drain(..) {
        mesh_state.transform_dirty.push(entity);
    }
    for entity in local_matrices_dirty.drain(..) {
        mesh_state.instanced_local_matrices_dirty.push(entity);
    }
    if needs_compute_dispatch {
        mesh_state.instanced_compute_dispatch_needed = true;
    }

    scratch.transform_dirty = transform_dirty;
    scratch.local_matrices_dirty = local_matrices_dirty;
    scratch.instanced_seen = seen;
}
