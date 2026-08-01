use nightshade::prelude::*;
use nightshade_graphics::prelude::*;
use nightshade_render::data::material::Material;
use nightshade_render::resources::material_registry::MaterialRegistry;

pub fn spawn_with_material(
    commands: &mut Commands,
    materials: &mut MaterialRegistry,
    mesh_name: &str,
    position: Vec3,
    scale: Vec3,
    material: Material,
) -> Entity {
    let entity = spawn_mesh_at(commands, mesh_name, position, scale);
    set_material(
        materials,
        commands,
        entity,
        format!("ManyLightsMaterial_{}", entity.id),
        material,
    );
    entity
}
