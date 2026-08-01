use nightshade_graphics::commands::loading::load_texture_pack_from_image_bytes;
use nightshade_graphics::resources::loading::TextureLoad;
pub use nightshade_render::data::material::solid_material;
use nightshade_render::data::material::{Material, TextureTransform};
use nightshade_render::data::texture_data::{SamplerSettings, TextureUsage};

pub const FLOOR_TEXTURE: &str = "proto_dark_06";

const PROTOTYPE_TEXTURES: &[(&str, &[u8])] = &[(
    FLOOR_TEXTURE,
    include_bytes!("../../../../../assets/textures/prototype/dark/texture_06.png") as &[u8],
)];

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
