use nightshade::ecs::loading::load_texture_pack_from_image_bytes;
use nightshade::ecs::material::components::{Material, TextureTransform};
use nightshade::prelude::*;
use nightshade::render::wgpu::texture_cache::{SamplerSettings, TextureUsage};

pub const FLOOR_TEXTURE: &str = "proto_dark_06";

const PROTOTYPE_TEXTURES: &[(&str, &[u8])] = &[(
    FLOOR_TEXTURE,
    include_bytes!("../../../../../assets/textures/prototype/dark/texture_06.png") as &[u8],
)];

pub fn load_prototype_textures(world: &mut World) {
    load_texture_pack_from_image_bytes(
        world,
        PROTOTYPE_TEXTURES,
        TextureUsage::Color,
        SamplerSettings::DEFAULT,
    );
}

pub fn prototype_material(texture: &str, tint: Vec3, roughness: f32, metallic: f32) -> Material {
    Material {
        base_color: [tint.x, tint.y, tint.z, 1.0],
        base_texture: Some(texture.to_string()),
        base_texture_transform: TextureTransform {
            scale: [4.0, 4.0],
            ..Default::default()
        },
        roughness,
        metallic,
        ..Default::default()
    }
}
