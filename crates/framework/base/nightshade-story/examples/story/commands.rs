use crate::theme;
use nalgebra_glm::{Vec3, vec3};
use nightshade::prelude::*;
use nightshade_graphics::prelude::*;
use nightshade_render::data::material::Material;
use nightshade_render::resources::material_registry::MaterialRegistry;

pub fn slab(
    commands: &mut Commands,
    materials: &mut MaterialRegistry,
    position: Vec3,
    scale: Vec3,
    tint: [f32; 4],
) -> Entity {
    let entity = spawn_mesh_at(commands, "Cube", position, scale);
    set_material(
        materials,
        commands,
        entity,
        format!("{}_{}", theme::TITLE, entity.id),
        Material {
            base_color: tint,
            roughness: 0.85,
            metallic: 0.0,
            ..Default::default()
        },
    );
    entity
}

pub fn tint(
    materials: &mut MaterialRegistry,
    commands: &mut Commands,
    entity: Entity,
    color: [f32; 4],
) {
    set_material(
        materials,
        commands,
        entity,
        format!("{}_{}", theme::TITLE, entity.id),
        Material {
            base_color: color,
            roughness: 0.85,
            metallic: 0.0,
            ..Default::default()
        },
    );
}

pub fn floor(commands: &mut Commands, materials: &mut MaterialRegistry, reach: f32) {
    slab(
        commands,
        materials,
        vec3(0.0, -0.5, 0.0),
        vec3(reach * 2.0, 1.0, reach * 2.0),
        theme::FLOOR_TINT,
    );
}
