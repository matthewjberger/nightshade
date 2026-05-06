use crate::ecs::material::resources::material_registry_insert;
use crate::ecs::world::{
    BOUNDING_VOLUME, CASTS_SHADOW, GLOBAL_TRANSFORM, LOCAL_TRANSFORM, LOCAL_TRANSFORM_DIRTY,
    MATERIAL_REF, NAME, Quat, RENDER_MESH, Vec3, World,
    components::{
        BoundingVolume, CastsShadow, GlobalTransform, LocalTransform, LocalTransformDirty,
        Material, MaterialRef, Name, RenderMesh,
    },
};
use freecs::Entity;

pub fn spawn_mesh(world: &mut World, mesh_name: &str, position: Vec3, scale: Vec3) -> Entity {
    let entity = world.spawn_entities(
        NAME | LOCAL_TRANSFORM
            | GLOBAL_TRANSFORM
            | LOCAL_TRANSFORM_DIRTY
            | RENDER_MESH
            | MATERIAL_REF
            | BOUNDING_VOLUME
            | CASTS_SHADOW,
        1,
    )[0];

    world.core.set_name(entity, Name(mesh_name.to_string()));
    world.core.set_local_transform(
        entity,
        LocalTransform {
            translation: position,
            scale,
            rotation: Quat::identity(),
        },
    );
    world
        .core
        .set_global_transform(entity, GlobalTransform::default());
    world
        .core
        .set_local_transform_dirty(entity, LocalTransformDirty);
    world
        .core
        .set_render_mesh(entity, RenderMesh::new(mesh_name));
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
        world
            .resources
            .assets
            .material_registry
            .registry
            .add_reference(index);
    }
    world
        .core
        .set_material_ref(entity, MaterialRef::new(material_name));
    world
        .core
        .set_bounding_volume(entity, BoundingVolume::from_mesh_type(mesh_name));
    world.core.set_casts_shadow(entity, CastsShadow);

    entity
}
