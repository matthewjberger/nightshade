use crate::theme::{HELMET_GLTF, HELMET_POSITION, ROOM};
use nightshade::prelude::*;
use nightshade_graphics::prelude::*;
use nightshade_render::resources::material_registry::MaterialRegistry;

pub fn spawn_room(registry: &mut MaterialRegistry, commands: &mut Commands<'_>) {
    for piece in &ROOM {
        let entity = spawn_mesh(commands, piece.mesh, piece.position, piece.scale);
        set_material(
            registry,
            commands,
            entity,
            format!("DecalDemoMaterial_{}", entity.id),
            Material {
                base_color: piece.base_color,
                roughness: piece.roughness,
                metallic: piece.metallic,
                ..Default::default()
            },
        );
    }
}

pub fn spawn_damaged_helmet(loading: &mut nightshade_gltf::Loading) {
    nightshade_load::load_noted(
        loading,
        nightshade_load::Source::Bytes(HELMET_GLTF.to_vec()),
        Some(HELMET_POSITION),
    );
}
