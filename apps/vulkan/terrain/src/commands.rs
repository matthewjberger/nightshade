use crate::components::TileKey;
use crate::queries::{boulder_mesh_name, tile_origin};
use crate::resources::Boulder;
use crate::theme;
use nightshade::prelude::*;
use nightshade_graphics::commands::builtins::set_mesh_material;
use nightshade_graphics::prelude::material_registry_insert;
use nightshade_render::commands::mesh_state::{
    mesh_state_mark_entity_added, mesh_state_mark_entity_removed,
};
use nightshade_render::components::culling::CastsShadow;
use nightshade_render::components::material::MaterialRef;
use nightshade_render::components::mesh::RenderMesh;
use nightshade_render::data::bounding_volume::{BoundingVolume, OrientedBoundingBox};
use nightshade_render::data::material::Material;
use nightshade_render::prelude::MeshRenderState;
use nightshade_render::resources::material_registry::MaterialRegistry;
use nightshade_scene::prelude::{GlobalTransform, LocalTransform, Name, Visibility};

pub fn hold_the_materials(registry: &mut MaterialRegistry) {
    material_registry_insert(
        registry,
        theme::GROUND_MATERIAL.to_string(),
        Material {
            base_color: theme::GROUND_COLOR,
            roughness: theme::GROUND_ROUGHNESS,
            metallic: 0.0,
            ..Default::default()
        },
    );
    material_registry_insert(
        registry,
        theme::ROCK_MATERIAL.to_string(),
        Material {
            base_color: theme::ROCK_COLOR,
            roughness: theme::ROCK_ROUGHNESS,
            metallic: 0.0,
            ..Default::default()
        },
    );
}

pub fn spawn_tile(
    commands: &mut Commands,
    key: [i32; 2],
    mesh_name: &str,
    lowest: f32,
    highest: f32,
) -> Entity {
    let transform = LocalTransform::from_translation(tile_origin(key));
    let lowest_corner = Vec3::new(0.0, lowest, 0.0);
    let highest_corner = Vec3::new(theme::TILE_SPAN, highest, theme::TILE_SPAN);
    let reach = (highest_corner - lowest_corner).norm() * 0.5;
    let entity = commands.spawn((
        Name(mesh_name.to_string()),
        transform,
        GlobalTransform(transform.as_matrix()),
        RenderMesh::new(mesh_name),
        MaterialRef::default(),
        Visibility { visible: true },
        BoundingVolume::new(
            OrientedBoundingBox::from_aabb(lowest_corner, highest_corner),
            reach,
        ),
    ));
    set_mesh_material(commands, entity, theme::GROUND_MATERIAL);
    set_mesh_material(commands, entity, theme::ROCK_MATERIAL);
    commands.set(entity, TileKey(key));
    commands.set(entity, CastsShadow);
    mark_added(commands, entity);
    entity
}

pub fn spawn_boulder(commands: &mut Commands, key: [i32; 2], boulder: Boulder) -> Entity {
    let mesh_name = boulder_mesh_name(boulder.kind);
    let transform = LocalTransform {
        translation: tile_origin(key) + boulder.place,
        rotation: nalgebra_glm::quat_angle_axis(boulder.yaw, &Vec3::y()),
        scale: Vec3::new(boulder.scale, boulder.scale, boulder.scale),
    };
    let reach = 1.0 + theme::LUMP_DEPTH + theme::GRAIN_DEPTH;
    let entity = commands.spawn((
        Name(mesh_name.clone()),
        transform,
        GlobalTransform(transform.as_matrix()),
        RenderMesh::new(&mesh_name),
        MaterialRef::default(),
        Visibility { visible: true },
        BoundingVolume::new(
            OrientedBoundingBox::from_aabb(
                Vec3::new(-reach, -reach, -reach),
                Vec3::new(reach, reach, reach),
            ),
            reach,
        ),
    ));
    set_mesh_material(commands, entity, theme::ROCK_MATERIAL);
    commands.set(entity, TileKey(key));
    commands.set(entity, CastsShadow);
    mark_added(commands, entity);
    entity
}

pub fn drop_the_entity(commands: &mut Commands, entity: Entity) {
    commands.queue(move |_, resources| {
        mesh_state_mark_entity_removed(resources.res_mut::<MeshRenderState>(), entity);
    });
    commands.despawn(entity);
}

fn mark_added(commands: &mut Commands, entity: Entity) {
    commands.queue(move |_, resources| {
        mesh_state_mark_entity_added(resources.res_mut::<MeshRenderState>(), entity);
    });
}
