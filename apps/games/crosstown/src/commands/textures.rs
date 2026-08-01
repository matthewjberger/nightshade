use crate::data;
use crate::queries::noise;
use crate::resources::Sheets;
use nightshade_graphics::commands::loading::queue_decoded_texture;
use nightshade_graphics::resources::loading::TextureLoad;
use nightshade_render::commands::material_registry::insert_material;
use nightshade_render::data::decoded_image::decode_to_rgba8;
use nightshade_render::data::material::Material;
use nightshade_render::data::texture_data::{
    SamplerFilter, SamplerSettings, SamplerWrap, TextureUsage,
};
use nightshade_render::resources::material_registry::MaterialRegistry;
use nightshade_sprite::prelude::{
    SpriteAtlases, SpriteFilter, sprite_atlas_create, sprite_atlas_insert,
};

pub fn pack_sheets(atlases: &mut SpriteAtlases) -> Sheets {
    sprite_atlas_create(atlases, data::ATLAS, data::ATLAS_SIZE, SpriteFilter::Linear);
    let pages = data::PAGE_NAMES
        .iter()
        .zip(data::PAGE_BYTES.iter())
        .map(|(name, bytes)| {
            let decoded = decode_to_rgba8(bytes).expect("the character sheet decodes");
            let (width, height) = (decoded.width, decoded.height);
            sprite_atlas_insert(atlases, data::ATLAS, *name, &decoded.rgba, width, height)
                .expect("the character sheet fits the atlas")
        })
        .collect();
    let shadow = sprite_atlas_insert(
        atlases,
        data::ATLAS,
        data::SHADOW,
        &blob(),
        data::SHADOW_TEXELS,
        data::SHADOW_TEXELS,
    )
    .expect("the shadow blob fits the atlas");
    let weapons = data::WEAPON_KINDS
        .iter()
        .map(|plan| {
            let (rgba, width, height) = weapon_picture(plan.name);
            sprite_atlas_insert(atlases, data::ATLAS, plan.sprite, &rgba, width, height)
                .expect("the weapon picture fits the atlas")
        })
        .collect();
    let pickups = data::PICKUP_KINDS
        .iter()
        .map(|plan| {
            let (rgba, width, height) = pickup_picture(plan.name);
            sprite_atlas_insert(atlases, data::ATLAS, plan.sprite, &rgba, width, height)
                .expect("the pickup picture fits the atlas")
        })
        .collect();
    Sheets {
        pages,
        shadow,
        weapons,
        pickups,
    }
}

fn weapon_picture(name: &str) -> (Vec<u8>, u32, u32) {
    let size = data::TEXEL * 2;
    let mut rgba = Vec::with_capacity((size * size * 4) as usize);
    let bottle = name == "BOTTLE";
    for row in 0..size {
        for column in 0..size {
            let across = column as f32 / size as f32;
            let down = row as f32 / size as f32;
            let solid = if bottle {
                let neck = down < 0.32 && (0.38..0.62).contains(&across);
                let body = down >= 0.32 && (0.2..0.8).contains(&across);
                neck || body
            } else {
                (0.34..0.66).contains(&down)
            };
            let shade = if down < 0.45 { 255 } else { 190 };
            let tone = if solid {
                [shade, shade, shade, 255]
            } else {
                [0, 0, 0, 0]
            };
            rgba.extend_from_slice(&tone);
        }
    }
    (rgba, size, size)
}

fn pickup_picture(name: &str) -> (Vec<u8>, u32, u32) {
    let size = data::TEXEL * 2;
    let half = size as f32 * 0.5;
    let mut rgba = Vec::with_capacity((size * size * 4) as usize);
    let roast = name == "ROAST";
    for row in 0..size {
        for column in 0..size {
            let x = (column as f32 + 0.5 - half) / half;
            let y = (row as f32 + 0.5 - half) / half;
            let tone = if roast {
                let inside = x * x * 1.25 + y * y < 0.82;
                let bone = y > 0.42 && x.abs() < 0.16;
                if bone {
                    [244, 238, 220, 255]
                } else if inside {
                    let shade = if y < -0.1 { 255 } else { 205 };
                    [shade, shade, shade, 255]
                } else {
                    [0, 0, 0, 0]
                }
            } else {
                let body = y > -0.62 && x.abs() < 0.46;
                let lid = (-0.78..=-0.62).contains(&y) && x.abs() < 0.5;
                let band = (-0.2..=0.06).contains(&y) && x.abs() < 0.46;
                if lid {
                    [232, 234, 240, 255]
                } else if band {
                    [248, 250, 255, 255]
                } else if body {
                    let shade = if x < -0.14 { 255 } else { 196 };
                    [shade, shade, shade, 255]
                } else {
                    [0, 0, 0, 0]
                }
            };
            rgba.extend_from_slice(&tone);
        }
    }
    (rgba, size, size)
}

pub fn load_textures(load: &mut TextureLoad<'_>) {
    let sampler = SamplerSettings {
        wrap_u: SamplerWrap::Repeat,
        wrap_v: SamplerWrap::Repeat,
        mag_filter: SamplerFilter::Nearest,
        min_filter: SamplerFilter::Nearest,
        mipmap_filter: SamplerFilter::Nearest,
    };
    for (name, rgba) in [
        (data::ASPHALT, speckled(data::ASPHALT_TONES, 11)),
        (data::CONCRETE, speckled(data::CONCRETE_TONES, 29)),
        (data::PAINT, speckled(data::PAINT_TONES, 47)),
        (data::METAL, speckled(data::METAL_TONES, 61)),
        (data::BRICK, bricks()),
        (data::WINDOW_DARK, pane(data::WINDOW_COLD)),
        (data::WINDOW_LIT, pane(data::WINDOW_WARM)),
        (data::WOOD, planks()),
        (data::TILE_WALL, tiles()),
        (data::GRAVEL, speckled(data::GRAVEL_TONES, 83)),
        (data::RUST, speckled(data::RUST_TONES, 97)),
        (data::RAIL, speckled(data::RAIL_TONES, 103)),
    ] {
        queue_decoded_texture(
            load,
            name.to_string(),
            rgba,
            data::TEXEL,
            data::TEXEL,
            TextureUsage::Color,
            sampler,
        );
    }
}

pub fn insert_materials(registry: &mut MaterialRegistry) {
    for (name, texture, roughness, metallic) in [
        (data::ASPHALT, data::ASPHALT, 0.6, 0.05),
        (data::CONCRETE, data::CONCRETE, 0.95, 0.0),
        (data::PAINT, data::PAINT, 0.8, 0.0),
        (data::BRICK, data::BRICK, 0.95, 0.0),
        (data::METAL, data::METAL, 0.45, 0.7),
        (data::WINDOW_DARK, data::WINDOW_DARK, 0.2, 0.2),
        (data::WOOD, data::WOOD, 0.9, 0.0),
        (data::TILE_WALL, data::TILE_WALL, 0.35, 0.0),
        (data::GRAVEL, data::GRAVEL, 1.0, 0.0),
        (data::RUST, data::RUST, 0.95, 0.15),
        (data::RAIL, data::RAIL, 0.5, 0.8),
    ] {
        insert_material(
            registry,
            name,
            Material {
                base_texture: Some(texture.to_string()),
                roughness,
                metallic,
                ..Default::default()
            },
        );
    }
    insert_material(
        registry,
        data::WINDOW_LIT,
        Material {
            base_texture: Some(data::WINDOW_LIT.to_string()),
            emissive_factor: data::WINDOW_GLOW,
            roughness: 0.4,
            ..Default::default()
        },
    );
    insert_material(
        registry,
        data::LAMP_GLASS,
        Material {
            base_color: [1.0, 0.9, 0.7, 1.0],
            emissive_factor: data::LAMP_GLOW,
            roughness: 0.3,
            ..Default::default()
        },
    );
}

fn speckled(tones: [[u8; 4]; 3], seed: u32) -> Vec<u8> {
    let mut rgba = Vec::with_capacity((data::TEXEL * data::TEXEL * 4) as usize);
    for index in 0..data::TEXEL * data::TEXEL {
        let pick = noise(index.wrapping_mul(seed).wrapping_add(seed));
        let tone = if pick < 0.2 {
            tones[1]
        } else if pick > 0.88 {
            tones[2]
        } else {
            tones[0]
        };
        rgba.extend_from_slice(&tone);
    }
    rgba
}

fn bricks() -> Vec<u8> {
    let mut rgba = Vec::with_capacity((data::TEXEL * data::TEXEL * 4) as usize);
    for row in 0..data::TEXEL {
        for column in 0..data::TEXEL {
            let course = row / 4;
            let shifted = (column + course * 4) % 8;
            let mortar = row % 4 == 0 || shifted == 0;
            let tone = if mortar {
                data::BRICK_MORTAR
            } else if noise(row * 31 + column * 7) < 0.35 {
                data::BRICK_ALT
            } else {
                data::BRICK_COLOR
            };
            rgba.extend_from_slice(&tone);
        }
    }
    rgba
}

fn planks() -> Vec<u8> {
    let mut rgba = Vec::with_capacity((data::TEXEL * data::TEXEL * 4) as usize);
    for row in 0..data::TEXEL {
        for column in 0..data::TEXEL {
            let seam = row % 5 == 0;
            let tone = if seam {
                data::WOOD_TONES[1]
            } else if noise(row * 17 + column * 3) < 0.3 {
                data::WOOD_TONES[2]
            } else {
                data::WOOD_TONES[0]
            };
            rgba.extend_from_slice(&tone);
        }
    }
    rgba
}

fn tiles() -> Vec<u8> {
    let mut rgba = Vec::with_capacity((data::TEXEL * data::TEXEL * 4) as usize);
    for row in 0..data::TEXEL {
        for column in 0..data::TEXEL {
            let grout = row % 8 == 0 || column % 8 == 0;
            let tone = if grout {
                data::TILE_TONES[1]
            } else if noise(row * 11 + column * 5) < 0.2 {
                data::TILE_TONES[2]
            } else {
                data::TILE_TONES[0]
            };
            rgba.extend_from_slice(&tone);
        }
    }
    rgba
}

fn pane(glass: [u8; 4]) -> Vec<u8> {
    let mut rgba = Vec::with_capacity((data::TEXEL * data::TEXEL * 4) as usize);
    for row in 0..data::TEXEL {
        for column in 0..data::TEXEL {
            let border = row < 2 || column < 2 || row > 13 || column > 13;
            let mullion = row == 7 || row == 8 || column == 7 || column == 8;
            let tone = if border || mullion {
                data::WINDOW_FRAME
            } else {
                glass
            };
            rgba.extend_from_slice(&tone);
        }
    }
    rgba
}

fn blob() -> Vec<u8> {
    let size = data::SHADOW_TEXELS as f32;
    let mut rgba = Vec::with_capacity((data::SHADOW_TEXELS * data::SHADOW_TEXELS * 4) as usize);
    for row in 0..data::SHADOW_TEXELS {
        for column in 0..data::SHADOW_TEXELS {
            let x = (column as f32 + 0.5) / size * 2.0 - 1.0;
            let y = (row as f32 + 0.5) / size * 2.0 - 1.0;
            let falloff = (1.0 - (x * x + y * y)).max(0.0);
            let alpha = (falloff * falloff * 255.0) as u8;
            rgba.extend_from_slice(&[255, 255, 255, alpha]);
        }
    }
    rgba
}
