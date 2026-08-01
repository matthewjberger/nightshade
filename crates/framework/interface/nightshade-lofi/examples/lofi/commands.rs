use crate::theme;
use nalgebra_glm::{Vec3, vec3};
use nightshade::prelude::*;
use nightshade_graphics::prelude::*;
use nightshade_render::data::material::Material;
use nightshade_render::resources::material_registry::MaterialRegistry;

pub fn block(
    commands: &mut Commands,
    materials: &mut MaterialRegistry,
    shape: &str,
    position: Vec3,
    scale: Vec3,
    tint: [f32; 4],
) -> Entity {
    let entity = spawn_mesh_at(commands, shape, position, scale);
    set_material(
        materials,
        commands,
        entity,
        format!("{}_{}", theme::TITLE, entity.id),
        Material {
            base_color: tint,
            roughness: 0.8,
            metallic: 0.0,
            ..Default::default()
        },
    );
    entity
}

pub fn floor(commands: &mut Commands, materials: &mut MaterialRegistry) {
    block(
        commands,
        materials,
        "Plane",
        vec3(0.0, 0.0, 0.0),
        vec3(theme::REACH, 1.0, theme::REACH),
        theme::FLOOR_TINT,
    );
}
