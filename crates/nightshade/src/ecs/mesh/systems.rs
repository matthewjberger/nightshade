use crate::ecs::world::{INSTANCED_MESH, World};

pub fn update_instanced_mesh_caches_system(world: &mut World) {
    let entities: Vec<_> = world.query_entities(INSTANCED_MESH).collect();
    let _span = tracing::info_span!("instanced_meshes", count = entities.len()).entered();

    for entity in entities {
        let parent_transform = world
            .get_global_transform(entity)
            .map(|t| t.0)
            .unwrap_or_else(nalgebra_glm::Mat4::identity);

        if let Some(instanced_mesh) = world.get_instanced_mesh_mut(entity) {
            instanced_mesh.update_world_matrices(&parent_transform);
        }
    }
}
