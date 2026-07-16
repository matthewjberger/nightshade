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

    world.set(entity, Name(mesh_name.to_string()));
    crate::ecs::world::commands::setup_entity_transforms(
        world,
        entity,
        LocalTransform {
            translation: position,
            scale,
            rotation: Quat::identity(),
        },
    );
    world.set(entity, RenderMesh::new(mesh_name));
    world
        .expect_resource_mut::<crate::render::mesh_state::MeshRenderState>()
        .mark_entity_added(render_entity(entity));
    let material_name = format!("Mesh_{}", entity.id);
    material_registry_insert(
        &mut world
            .expect_resource_mut::<crate::ecs::asset_state::AssetState>()
            .material_registry,
        material_name.clone(),
        Material::default(),
    );
    if let Some(&index) = world
        .expect_resource_mut::<crate::ecs::asset_state::AssetState>()
        .material_registry
        .registry
        .name_to_index
        .get(&material_name)
    {
        registry_add_reference(
            &mut world
                .expect_resource_mut::<crate::ecs::asset_state::AssetState>()
                .material_registry
                .registry,
            index,
        );
    }
    world.set(entity, MaterialRef::new(material_name));
    world.set(entity, BoundingVolume::from_mesh_type(mesh_name));
    world.set(entity, CastsShadow);

    entity
}
