use crate::data;
use nightshade_graphics::commands::loading::queue_decoded_texture;
use nightshade_graphics::resources::loading::TextureLoad;
use nightshade_render::commands::texture_cache::texture_cache_protect;
use nightshade_render::data::texture_data::{SamplerSettings, TextureUsage};

pub fn dress(mut textures: TextureLoad) {
    for (name, pixels) in [
        (data::TILE_TEXTURE, tiles()),
        (data::GRID_TEXTURE, grid()),
        (data::SPOT_TEXTURE, spot()),
    ] {
        queue_decoded_texture(
            &mut textures,
            name.to_string(),
            pixels,
            data::SKIN_SIDE,
            data::SKIN_SIDE,
            TextureUsage::Color,
            SamplerSettings::DEFAULT,
        );
        texture_cache_protect(textures.cache, name.to_string());
    }
}

fn tiles() -> Vec<u8> {
    let side = data::SKIN_SIDE;
    let mut pixels = Vec::with_capacity((side * side * 4) as usize);
    for row in 0..side {
        for column in 0..side {
            let edge = row < 2 || column < 2 || row + 2 >= side || column + 2 >= side;
            let grain = ((row * 7 + column * 13) % 11) as f32 / 11.0;
            let shade = if edge { 0.34 } else { 0.72 + grain * 0.18 };
            let level = (shade * 255.0) as u8;
            pixels.extend_from_slice(&[level, level, level, 255]);
        }
    }
    pixels
}

fn grid() -> Vec<u8> {
    let side = data::SKIN_SIDE;
    let mut pixels = Vec::with_capacity((side * side * 4) as usize);
    for row in 0..side {
        for column in 0..side {
            let line = row < 1 || column < 1;
            let level = if line { 230 } else { 40 };
            pixels.extend_from_slice(&[level, level, level, 255]);
        }
    }
    pixels
}

fn spot() -> Vec<u8> {
    let side = data::SKIN_SIDE;
    let middle = side as f32 * 0.5;
    let mut pixels = Vec::with_capacity((side * side * 4) as usize);
    for row in 0..side {
        for column in 0..side {
            let reach = ((column as f32 - middle).hypot(row as f32 - middle)) / middle;
            let alpha = (1.0 - reach).clamp(0.0, 1.0).powf(1.4);
            pixels.extend_from_slice(&[255, 255, 255, (alpha * 255.0) as u8]);
        }
    }
    pixels
}
