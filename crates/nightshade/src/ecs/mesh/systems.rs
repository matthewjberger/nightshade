use crate::ecs::world::{GLOBAL_TRANSFORM, INSTANCED_MESH, World};

pub fn update_instanced_mesh_caches_system(world: &mut World) {
    let _span = tracing::info_span!("instanced_meshes").entered();

    let mut transform_dirty = std::mem::take(&mut world.resources.scratch.transform_dirty);
    let mut local_matrices_dirty =
        std::mem::take(&mut world.resources.scratch.local_matrices_dirty);
    transform_dirty.clear();
    local_matrices_dirty.clear();
    let mut needs_compute_dispatch = false;

    world
        .core
        .query_mut()
        .with(INSTANCED_MESH)
        .iter(|entity, table, idx| {
            let parent_transform = if table.mask & GLOBAL_TRANSFORM != 0 {
                table.global_transform[idx].0
            } else {
                nalgebra_glm::Mat4::identity()
            };
            let instanced_mesh = &mut table.instanced_mesh[idx];
            let was_dirty = instanced_mesh.dirty;
            let had_gpu_dirty = instanced_mesh.local_matrices_gpu_dirty();
            let parent_only_changed = instanced_mesh.update_world_matrices(&parent_transform);
            let custom_data_dirty = !instanced_mesh.take_dirty_custom_data_indices().is_empty();
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

    for entity in transform_dirty.drain(..) {
        world
            .resources
            .mesh_render_state
            .mark_transform_dirty(entity);
    }
    for entity in local_matrices_dirty.drain(..) {
        world
            .resources
            .mesh_render_state
            .mark_instanced_local_matrices_dirty(entity);
    }
    if needs_compute_dispatch {
        world
            .resources
            .mesh_render_state
            .mark_instanced_compute_dispatch_needed();
    }

    world.resources.scratch.transform_dirty = transform_dirty;
    world.resources.scratch.local_matrices_dirty = local_matrices_dirty;
}
