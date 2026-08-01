use nightshade_graphics::commands::loading::load_texture_pack_from_image_bytes;
use nightshade_graphics::resources::loading::TextureLoad;
pub use nightshade_render::data::material::solid_material;
use nightshade_render::data::material::{Material, TextureTransform};
use nightshade_render::data::texture_data::{SamplerSettings, TextureUsage};

pub const GROUND_GREEN: &str = "ground_green";
pub const GROUND_ORANGE: &str = "ground_orange";
pub const GROUND_DARK: &str = "ground_dark";

const PROTOTYPE_TEXTURES: &[(&str, &[u8])] = &[
    (
        GROUND_GREEN,
        include_bytes!("../../../../../assets/textures/prototype/green/texture_04.png") as &[u8],
    ),
    (
        GROUND_ORANGE,
        include_bytes!("../../../../../assets/textures/prototype/orange/texture_04.png") as &[u8],
    ),
    (
        GROUND_DARK,
        include_bytes!("../../../../../assets/textures/prototype/dark/texture_06.png") as &[u8],
    ),
];

pub fn load_prototype_textures(load: &mut TextureLoad<'_>) {
    load_texture_pack_from_image_bytes(
        load,
        PROTOTYPE_TEXTURES,
        TextureUsage::Color,
        SamplerSettings::DEFAULT,
    );
}

pub fn tiled_material(texture: &str, tiling: f32) -> Material {
    Material {
        base_color: [1.0, 1.0, 1.0, 1.0],
        base_texture: Some(texture.to_string()),
        base_texture_transform: TextureTransform {
            scale: [tiling, tiling],
            ..Default::default()
        },
        roughness: 0.85,
        metallic: 0.0,
        ..Default::default()
    }
}
