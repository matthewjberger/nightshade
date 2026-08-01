use crate::data::{LEDGE_TEXTURE, STONE_TEXTURE};
use nightshade_graphics::commands::loading::load_texture_pack_from_image_bytes;
use nightshade_graphics::resources::loading::TextureLoad;
pub use nightshade_render::data::material::solid_material;
use nightshade_render::data::material::{Material, TextureTransform};
use nightshade_render::data::texture_data::{SamplerSettings, TextureUsage};

const SURFACES: &[(&str, &[u8])] = &[
    (
        LEDGE_TEXTURE,
        include_bytes!("../../../../../assets/textures/prototype/orange/texture_04.png") as &[u8],
    ),
    (
        STONE_TEXTURE,
        include_bytes!("../../../../../assets/textures/prototype/dark/texture_06.png") as &[u8],
    ),
];

pub fn load_surfaces(load: &mut TextureLoad<'_>) {
    load_texture_pack_from_image_bytes(
        load,
        SURFACES,
        TextureUsage::Color,
        SamplerSettings::DEFAULT,
    );
}

pub fn tiled_material(texture: &str, tiling: [f32; 2]) -> Material {
    Material {
        base_color: [1.0, 1.0, 1.0, 1.0],
        base_texture: Some(texture.to_string()),
        base_texture_transform: TextureTransform {
            scale: tiling,
            ..Default::default()
        },
        roughness: 0.88,
        metallic: 0.0,
        ..Default::default()
    }
}

pub fn glowing_material(color: [f32; 4], glow: [f32; 3]) -> Material {
    Material {
        base_color: color,
        emissive_factor: glow,
        roughness: 0.35,
        metallic: 0.0,
        ..Default::default()
    }
}
