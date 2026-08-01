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
            roughness: 0.9,
            metallic: 0.0,
            ..Default::default()
        },
    );
    entity
}

pub fn column(
    commands: &mut Commands,
    materials: &mut MaterialRegistry,
    across: f32,
    along: f32,
    height: f32,
    tint: [f32; 4],
) -> Entity {
    slab(
        commands,
        materials,
        vec3(across, height * 0.5, along),
        vec3(theme::STEP, height.max(0.2), theme::STEP),
        tint,
    )
}
