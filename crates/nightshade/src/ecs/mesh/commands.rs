use crate::ecs::world::{
    BOUNDING_VOLUME, CASTS_SHADOW, GLOBAL_TRANSFORM, LOCAL_TRANSFORM, LOCAL_TRANSFORM_DIRTY,
    MATERIAL, NAME, Quat, RENDER_MESH, Vec3, World,
    components::{
        BoundingVolume, CastsShadow, GlobalTransform, LocalTransform, LocalTransformDirty,
        Material, Name, RenderMesh,
    },
};
use freecs::Entity;

pub fn spawn_mesh(world: &mut World, mesh_name: &str, position: Vec3, scale: Vec3) -> Entity {
    let entity = world.spawn_entities(
        NAME | LOCAL_TRANSFORM
            | GLOBAL_TRANSFORM
            | LOCAL_TRANSFORM_DIRTY
            | RENDER_MESH
            | MATERIAL
            | BOUNDING_VOLUME
            | CASTS_SHADOW,
        1,
    )[0];

    world.set_name(entity, Name(mesh_name.to_string()));
    world.set_local_transform(
        entity,
        LocalTransform {
            translation: position,
            scale,
            rotation: Quat::identity(),
        },
    );
    world.set_global_transform(entity, GlobalTransform::default());
    world.set_local_transform_dirty(entity, LocalTransformDirty);
    world.set_render_mesh(
        entity,
        RenderMesh {
            name: mesh_name.to_string(),
        },
    );
    world.set_material(entity, Material::default());
    world.set_bounding_volume(entity, BoundingVolume::from_mesh_type(mesh_name));
    world.set_casts_shadow(entity, CastsShadow);

    entity
}
