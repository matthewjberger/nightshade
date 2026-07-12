use nightshade::ecs::loading::load_texture_pack_from_image_bytes;
use nightshade::prelude::*;
use nightshade::render::material::{Material, TextureTransform};
use nightshade::render::wgpu::texture_cache::{SamplerSettings, TextureUsage};

pub const FLOOR_TEXTURE: &str = "proto_dark_06";
pub const STAIRS_TEXTURE: &str = "proto_green_01";
pub const RAMP_TEXTURE: &str = "proto_orange_01";
pub const PLATFORM_TEXTURE: &str = "proto_light_01";
pub const OBSTACLE_TEXTURE: &str = "proto_purple_01";

const PROTOTYPE_TEXTURES: &[(&str, &[u8])] = &[
    (
        FLOOR_TEXTURE,
        include_bytes!("../../../../../assets/textures/prototype/dark/texture_06.png") as &[u8],
    ),
    (
        STAIRS_TEXTURE,
        include_bytes!("../../../../../assets/textures/prototype/green/texture_01.png") as &[u8],
    ),
    (
        RAMP_TEXTURE,
        include_bytes!("../../../../../assets/textures/prototype/orange/texture_01.png") as &[u8],
    ),
    (
        PLATFORM_TEXTURE,
        include_bytes!("../../../../../assets/textures/prototype/light/texture_01.png") as &[u8],
    ),
    (
        OBSTACLE_TEXTURE,
        include_bytes!("../../../../../assets/textures/prototype/purple/texture_01.png") as &[u8],
    ),
];

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
