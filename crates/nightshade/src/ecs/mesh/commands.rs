use crate::ecs::material::resources::material_registry_insert;
use crate::ecs::world::{
    BOUNDING_VOLUME, CASTS_SHADOW, GLOBAL_TRANSFORM, LOCAL_TRANSFORM, MATERIAL_REF, NAME, Quat,
    RENDER_MESH, Vec3, World,
    components::{
        BoundingVolume, CastsShadow, LocalTransform, Material, MaterialRef, Name, RenderMesh,
    },
};
use freecs::Entity;

use crate::prelude::*;
pub fn spawn_mesh(world: &mut World, mesh_name: &str, position: Vec3, scale: Vec3) -> Entity {
    let entity = spawn_entities(
        world,
        NAME | LOCAL_TRANSFORM
            | GLOBAL_TRANSFORM
            | RENDER_MESH
            | MATERIAL_REF
            | BOUNDING_VOLUME
            | CASTS_SHADOW,
        1,
    )[0];

    world.ecs.worlds[CORE].set(entity, Name(mesh_name.to_string()));
    crate::ecs::world::commands::setup_entity_transforms(
        world,
        entity,
        LocalTransform {
            translation: position,
            scale,
            rotation: Quat::identity(),
        },
    );
    world.ecs.worlds[CORE].set(entity, RenderMesh::new(mesh_name));
    world.resources.mesh_render_state.mark_entity_added(entity);
    let material_name = format!("Mesh_{}", entity.id);
    material_registry_insert(
        &mut world.resources.assets.material_registry,
        material_name.clone(),
        Material::default(),
    );
    if let Some(&index) = world
        .resources
        .assets
        .material_registry
        .registry
        .name_to_index
        .get(&material_name)
    {
        registry_add_reference(
            &mut world.resources.assets.material_registry.registry,
            index,
        );
    }
    world.ecs.worlds[CORE].set(entity, MaterialRef::new(material_name));
    world.ecs.worlds[CORE].set(entity, BoundingVolume::from_mesh_type(mesh_name));
    world.ecs.worlds[CORE].set(entity, CastsShadow);

    entity
}
