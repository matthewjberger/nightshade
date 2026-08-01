use crate::data;
use nightshade_graphics::prelude::MaterialRegistry;
use nightshade_render::commands::material_registry::insert_material;
use nightshade_render::data::material::Material;
use nightshade_render::data::material::solid_material;

pub fn insert_all(registry: &mut MaterialRegistry) {
    insert_material(
        registry,
        data::STONE,
        solid_material(data::STONE_COLOR, 0.94, 0.0),
    );
    insert_material(
        registry,
        data::WALL,
        solid_material(data::WALL_COLOR, 0.9, 0.0),
    );
    insert_material(
        registry,
        data::PILLAR,
        solid_material(data::PILLAR_COLOR, 0.82, 0.0),
    );
    insert_material(registry, data::SEAL, glow(data::SEAL_COLOR, 1.1));
    insert_material(
        registry,
        data::PLATE,
        solid_material(data::PLATE_COLOR, 0.6, 0.0),
    );
    insert_material(
        registry,
        data::GLASS_GROUND,
        solid_material(data::GLASS_GROUND_COLOR, 0.9, 0.0),
    );
    insert_material(
        registry,
        data::GLASS_LEDGE,
        solid_material(data::GLASS_LEDGE_COLOR, 0.66, 0.0),
    );
    insert_material(
        registry,
        data::GLASS_WALL,
        solid_material(data::GLASS_WALL_COLOR, 0.95, 0.0),
    );
    insert_material(registry, data::SHUT_ARCH, glow(data::SHUT_COLOR, 2.0));
    insert_material(registry, data::PIP_LIT, glow(data::PIP_COLOR, 2.6));
    insert_material(registry, data::RUNNER, glow(data::RUNNER_COLOR, 1.4));
    insert_material(registry, data::BONE, tone(data::BONE_COLOR));
    insert_material(registry, data::AMBER, tone(data::AMBER_COLOR));
    insert_material(registry, data::AZURE, tone(data::AZURE_COLOR));
    insert_material(registry, data::MOSS, tone(data::MOSS_COLOR));
    insert_material(registry, data::PORTAL_MARK, glow(data::PORTAL_RIM, 1.6));
    for mark in data::ART_MARKS {
        if let Some(color) = data::art_color(mark) {
            insert_material(registry, data::art_material(mark), tone(color));
        }
    }
}

fn tone(color: [f32; 4]) -> Material {
    Material {
        base_color: color,
        emissive_factor: [0.0, 0.0, 0.0],
        emissive_strength: 0.0,
        roughness: 0.5,
        metallic: 0.0,
        unlit: true,
        ..Default::default()
    }
}

fn glow(color: [f32; 4], emissive: f32) -> Material {
    Material {
        base_color: color,
        emissive_factor: [color[0], color[1], color[2]],
        emissive_strength: emissive,
        roughness: 0.3,
        metallic: 0.0,
        unlit: true,
        ..Default::default()
    }
}
