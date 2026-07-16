use crate::ecs::world::CORE;
use crate::ecs::world::{GLOBAL_TRANSFORM, INSTANCED_MESH, World};
use crate::render_driver::render_entity;

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
    transform_dirty.clear();
    local_matrices_dirty.clear();
    let mut needs_compute_dispatch = false;

    let instanced_key =
        world.ecs.worlds[CORE].register::<crate::ecs::mesh::components::InstancedMesh>();
    let global_key =
        world.ecs.worlds[CORE].register::<crate::ecs::transform::components::GlobalTransform>();
    world.ecs.worlds[CORE].for_each_mut(INSTANCED_MESH, 0, |entity, table, idx| {
        let parent_transform = if table.has_component(GLOBAL_TRANSFORM) {
            table.column(global_key)[idx].0
        } else {
            nalgebra_glm::Mat4::identity()
        };
        let instanced_mesh = &mut table.column_mut(instanced_key)[idx];
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
            .res_mut::<crate::render::mesh_state::MeshRenderState>()
            .mark_transform_dirty(render_entity(entity));
    }
    for entity in local_matrices_dirty.drain(..) {
        world
            .res_mut::<crate::render::mesh_state::MeshRenderState>()
            .mark_instanced_local_matrices_dirty(render_entity(entity));
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
}
