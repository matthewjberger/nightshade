use crate::theme;
use nalgebra_glm::Vec3;
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
