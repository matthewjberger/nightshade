use crate::data;
use crate::theme;
use nalgebra_glm::{Vec3, vec3};
use nightshade::prelude::*;
use nightshade_graphics::prelude::*;
use nightshade_render::data::material::Material;
use nightshade_render::resources::material_registry::MaterialRegistry;

pub fn spawn_slab(
    commands: &mut Commands,
    materials: &mut MaterialRegistry,
    name: &str,
    position: Vec3,
    scale: Vec3,
    tint: [f32; 4],
) -> Entity {
    let entity = spawn_mesh_at(commands, "Cube", position, scale);
    set_material(
        materials,
        commands,
        entity,
        format!("{name}_{}", entity.id),
        Material {
            base_color: tint,
            roughness: 0.85,
            metallic: 0.0,
            ..Default::default()
        },
    );
    entity
}

pub fn spawn_grazer(
    commands: &mut Commands,
    materials: &mut MaterialRegistry,
    home: Vec3,
) -> Entity {
    let entity = spawn_slab(
        commands,
        materials,
        data::GRAZER,
        home,
        theme::GRAZER_SCALE,
        theme::GRAZER_TINT,
    );
    commands.queue(move |storage, _resources| {
        storage.set(entity, nightshade_grounded::Grounded(theme::GRAZER_HEIGHT));
        storage.set(entity, nightshade_wander::Home(home));
        storage.set(entity, nightshade_wander::Wandering(home));
        storage.set(entity, nightshade_wander::Resting(0.0));
        storage.set(entity, nightshade_wander::Roaming(theme::ROAMING));
        storage.set(entity, nightshade_health::Health(theme::GRAZER_HEALTH));
        storage.set(entity, nightshade_health::MaxHealth(theme::GRAZER_HEALTH));
        storage.set(entity, nightshade_health::Shield(0.0));
        storage.set(entity, nightshade_health::Armor(theme::GRAZER_ARMOR));
        storage.set(entity, nightshade_health::PendingDamage(0.0));
        storage.set(entity, nightshade_health::Bounty(1));
    });
    entity
}

pub fn ring_spot(index: usize, count: usize, reach: f32) -> Vec3 {
    let turn = (index as f32 / count as f32) * std::f32::consts::TAU;
    vec3(turn.cos() * reach, 0.0, turn.sin() * reach)
}
