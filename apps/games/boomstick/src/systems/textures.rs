use nightshade_graphics::commands::loading::load_texture_pack_from_image_bytes;
use nightshade_graphics::resources::loading::TextureLoad;
use nightshade_render::data::material::{Material, TextureTransform};
use nightshade_render::data::texture_data::{
    SamplerFilter, SamplerSettings, SamplerWrap, TextureUsage,
};

pub const GRATE: &str = "boomstick_grate";
pub const DECK: &str = "boomstick_deck";
pub const BRICK: &str = "boomstick_brick";
pub const STONE: &str = "boomstick_stone";
pub const RUST: &str = "boomstick_rust";
pub const HATCH: &str = "boomstick_hatch";
pub const SIGN: &str = "boomstick_sign";

const TEXTURES: &[(&str, &[u8])] = &[
    (GRATE, include_bytes!("../../assets/textures/floor.png")),
    (DECK, include_bytes!("../../assets/textures/courtyard.png")),
    (BRICK, include_bytes!("../../assets/textures/wall.png")),
    (STONE, include_bytes!("../../assets/textures/ledge.png")),
    (RUST, include_bytes!("../../assets/textures/metal.png")),
    (HATCH, include_bytes!("../../assets/textures/door.png")),
    (SIGN, include_bytes!("../../assets/textures/trim.png")),
];

pub fn load(load: &mut TextureLoad<'_>) {
    let sampler = SamplerSettings {
        wrap_u: SamplerWrap::Repeat,
        wrap_v: SamplerWrap::Repeat,
        mag_filter: SamplerFilter::Linear,
        min_filter: SamplerFilter::Linear,
        mipmap_filter: SamplerFilter::Linear,
    };
    load_texture_pack_from_image_bytes(load, TEXTURES, TextureUsage::Color, sampler);
}

pub fn tiles(run: f32, across: f32, per_unit: f32) -> [f32; 2] {
    [(run * per_unit).max(0.5), (across * per_unit).max(0.5)]
}

pub fn tiled(texture: &str, tiling: [f32; 2], tint: [f32; 4]) -> Material {
    Material {
        base_color: tint,
        base_texture: Some(texture.to_string()),
        base_texture_transform: TextureTransform {
            scale: tiling,
            ..Default::default()
        },
        roughness: 0.92,
        metallic: 0.0,
        ..Default::default()
    }
}

pub fn glowing(texture: &str, tiling: [f32; 2], tint: [f32; 4], glow: [f32; 3]) -> Material {
    Material {
        base_color: tint,
        base_texture: Some(texture.to_string()),
        base_texture_transform: TextureTransform {
            scale: tiling,
            ..Default::default()
        },
        emissive_factor: glow,
        roughness: 0.62,
        metallic: 0.08,
        ..Default::default()
    }
}
