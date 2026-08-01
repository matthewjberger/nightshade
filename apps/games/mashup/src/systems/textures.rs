use crate::data;
use nightshade_graphics::commands::loading::queue_decoded_texture;
use nightshade_graphics::resources::loading::TextureLoad;
use nightshade_render::commands::material_registry::insert_material;
use nightshade_render::commands::texture_cache::texture_cache_protect;
use nightshade_render::data::material::Material;
use nightshade_render::data::texture_data::{
    SamplerFilter, SamplerSettings, SamplerWrap, TextureUsage,
};
use nightshade_render::resources::material_registry::MaterialRegistry;

pub const BLOCK_TEXTURE: &str = "mashup_block_atlas";
pub const BLOCK_MATERIAL: &str = "mashup_block";

pub const GRASS_TOP: u32 = 0;
pub const GRASS_SIDE: u32 = 1;
pub const DIRT: u32 = 2;
pub const STONE: u32 = 3;

pub fn paint_art(art: &[&str]) -> (Vec<u8>, u32, u32) {
    let height = art.len() as u32;
    let width = art.first().map(|row| row.chars().count()).unwrap_or(0) as u32;
    let mut pixels = vec![0_u8; (width * height * 4) as usize];
    for (row, line) in art.iter().enumerate() {
        for (column, mark) in line.chars().enumerate() {
            let Some(color) = data::art_color(mark) else {
                continue;
            };
            let offset = ((row as u32 * width + column as u32) * 4) as usize;
            for channel in 0..3 {
                pixels[offset + channel] = (color[channel] * 255.0).round() as u8;
            }
            pixels[offset + 3] = 255;
        }
    }
    (pixels, width, height)
}

pub fn shadow_pixels() -> (Vec<u8>, u32, u32) {
    let size = data::TILE_TEXELS;
    let mut pixels = vec![0_u8; (size * size * 4) as usize];
    let middle = (size as f32 - 1.0) * 0.5;
    for row in 0..size {
        for column in 0..size {
            let x = (column as f32 - middle) / middle;
            let y = (row as f32 - middle) / middle;
            let reach = (x * x + y * y).sqrt();
            let strength = (1.0 - reach).clamp(0.0, 1.0);
            let offset = ((row * size + column) * 4) as usize;
            pixels[offset + 3] = (strength * strength * 255.0).round() as u8;
        }
    }
    (pixels, size, size)
}

pub fn block_atlas() -> (Vec<u8>, u32, u32) {
    let size = data::BLOCK_ATLAS_SIZE;
    let tile = data::TILE_TEXELS;
    let mut pixels = vec![255_u8; (size * size * 4) as usize];
    for cell in 0..4_u32 {
        for row in 0..tile {
            for column in 0..tile {
                let grain = data::hash_unit((cell * 64 + column) as i32, row as i32);
                let shade = 0.86 + grain * 0.28;
                let mut color = match cell {
                    GRASS_TOP => [0.30, 0.56, 0.26],
                    GRASS_SIDE => {
                        if row < 4 {
                            [0.30, 0.56, 0.26]
                        } else {
                            [0.46, 0.34, 0.22]
                        }
                    }
                    DIRT => [0.44, 0.32, 0.21],
                    _ => [0.52, 0.52, 0.56],
                };
                for channel in &mut color {
                    *channel = (*channel * shade).clamp(0.0, 1.0);
                }
                let x = cell * tile + column;
                let offset = ((row * size + x) * 4) as usize;
                for channel in 0..3 {
                    pixels[offset + channel] = (color[channel] * 255.0).round() as u8;
                }
                pixels[offset + 3] = 255;
            }
        }
    }
    (pixels, size, size)
}

pub fn cell_bounds(cell: u32) -> ([f32; 2], [f32; 2]) {
    let size = data::BLOCK_ATLAS_SIZE as f32;
    let tile = data::TILE_TEXELS as f32;
    let inset = 0.5 / size;
    let low = cell as f32 * tile / size;
    let high = (cell as f32 + 1.0) * tile / size;
    ([low + inset, inset], [high - inset, tile / size - inset])
}

pub fn load_block_atlas(load: &mut TextureLoad<'_>) {
    let (pixels, width, height) = block_atlas();
    let sampler = SamplerSettings {
        wrap_u: SamplerWrap::ClampToEdge,
        wrap_v: SamplerWrap::ClampToEdge,
        mag_filter: SamplerFilter::Nearest,
        min_filter: SamplerFilter::Nearest,
        mipmap_filter: SamplerFilter::Nearest,
    };
    queue_decoded_texture(
        load,
        BLOCK_TEXTURE.to_string(),
        pixels,
        width,
        height,
        TextureUsage::Color,
        sampler,
    );
    texture_cache_protect(load.cache, BLOCK_TEXTURE.to_string());
}

pub fn insert_materials(registry: &mut MaterialRegistry) {
    insert_material(
        registry,
        BLOCK_MATERIAL,
        Material {
            base_color: [1.0, 1.0, 1.0, 1.0],
            base_texture: Some(BLOCK_TEXTURE.to_string()),
            roughness: 1.0,
            metallic: 0.0,
            ..Default::default()
        },
    );
}
