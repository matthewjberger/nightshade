use nightshade::prelude::*;
use nightshade_graphics::commands::loading::queue_decoded_texture;
use nightshade_graphics::prelude::*;
use nightshade_graphics::resources::loading::TextureLoad;
use nightshade_render::commands::texture_cache::texture_cache_protect;
use nightshade_render::data::material::TextureTransform;
use nightshade_render::data::texture_data::{SamplerSettings, TextureUsage};

use crate::commands::ground_texture;
use crate::data;

pub fn lay_the_ground(
    mut textures: TextureLoad,
    mut registry: ResMut<nightshade_render::resources::material_registry::MaterialRegistry>,
    mut commands: Commands,
) {
    queue_decoded_texture(
        &mut textures,
        data::GROUND_TEXTURE.to_string(),
        ground_texture::pixels(),
        data::GROUND_TEXTURE_SIDE,
        data::GROUND_TEXTURE_SIDE,
        TextureUsage::Color,
        SamplerSettings::DEFAULT,
    );
    texture_cache_protect(textures.cache, data::GROUND_TEXTURE.to_string());

    let ground = spawn_mesh(
        &mut commands,
        "Plane",
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(data::GROUND_EXTENT, 1.0, data::GROUND_EXTENT),
    );
    set_material(
        &mut registry,
        &mut commands,
        ground,
        "ground".to_string(),
        Material {
            base_color: [1.0, 1.0, 1.0, 1.0],
            base_texture: Some(data::GROUND_TEXTURE.to_string()),
            base_texture_transform: TextureTransform {
                scale: [data::GROUND_TILES, data::GROUND_TILES],
                ..Default::default()
            },
            roughness: 0.95,
            metallic: 0.0,
            ..Default::default()
        },
    );
}
