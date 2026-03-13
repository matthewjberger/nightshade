use crate::ecs::world::{INSTANCED_MESH, World};

pub fn update_instanced_mesh_caches_system(world: &mut World) {
    let entities: Vec<_> = world.core.query_entities(INSTANCED_MESH).collect();
    let _span = tracing::info_span!("instanced_meshes", count = entities.len()).entered();

    let mut transform_dirty_entities = Vec::new();
    let mut local_matrices_dirty_entities = Vec::new();
    let mut needs_compute_dispatch = false;
    for entity in entities {
        let parent_transform = world
            .core
            .get_global_transform(entity)
            .map(|t| t.0)
            .unwrap_or_else(nalgebra_glm::Mat4::identity);

        if let Some(instanced_mesh) = world.core.get_instanced_mesh_mut(entity) {
            let was_dirty = instanced_mesh.dirty;
            let had_gpu_dirty = instanced_mesh.local_matrices_gpu_dirty();
            let parent_only_changed = instanced_mesh.update_world_matrices(&parent_transform);
            instanced_mesh.take_dirty_custom_data_indices();
            if was_dirty {
                transform_dirty_entities.push(entity);
            }
            if had_gpu_dirty {
                local_matrices_dirty_entities.push(entity);
                instanced_mesh.clear_local_matrices_gpu_dirty();
            }
            if parent_only_changed {
                needs_compute_dispatch = true;
            }
        }
    }

    for entity in transform_dirty_entities {
        world
            .resources
            .mesh_render_state
            .mark_transform_dirty(entity);
    }
    for entity in local_matrices_dirty_entities {
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
}
