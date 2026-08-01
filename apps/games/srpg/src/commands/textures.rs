use crate::resources::Sheet;
use crate::theme;
use fastrand::Rng;
use nightshade_graphics::commands::loading::queue_decoded_texture;
use nightshade_graphics::resources::loading::TextureLoad;
use nightshade_render::commands::material_registry::insert_material;
use nightshade_render::data::material::{AlphaMode, Material};
use nightshade_render::data::texture_data::{
    SamplerFilter, SamplerSettings, SamplerWrap, TextureUsage,
};
use nightshade_render::resources::material_registry::MaterialRegistry;
use nightshade_sprite::commands::sheet::tile_trim;
use nightshade_sprite::prelude::*;

pub fn load_block_textures(load: &mut TextureLoad<'_>, rgba: &[u8], width: u32) {
    let sampler = SamplerSettings {
        wrap_u: SamplerWrap::Repeat,
        wrap_v: SamplerWrap::Repeat,
        mag_filter: SamplerFilter::Nearest,
        min_filter: SamplerFilter::Nearest,
        mipmap_filter: SamplerFilter::Nearest,
    };
    let textures = [
        (
            theme::GRASS_TEXTURE,
            tile_pixels(
                rgba,
                width,
                theme::TILE_TEXELS,
                theme::SHEET_COLUMNS,
                theme::GRASS_GROUND,
            ),
        ),
        (
            theme::DIRT_TEXTURE,
            speckled(
                theme::DIRT_COLOR,
                theme::DIRT_DARK,
                theme::DIRT_LIGHT,
                23,
                20,
                14,
            ),
        ),
        (
            theme::SAND_TEXTURE,
            speckled(
                theme::SAND_COLOR,
                theme::SAND_DARK,
                theme::SAND_LIGHT,
                37,
                14,
                14,
            ),
        ),
        (
            theme::STONE_TEXTURE,
            speckled(
                theme::ROAD_COLOR,
                theme::ROAD_DARK,
                theme::ROAD_LIGHT,
                53,
                24,
                16,
            ),
        ),
    ];
    for (name, rgba) in textures {
        queue_decoded_texture(
            load,
            name.to_string(),
            rgba,
            theme::BLOCK_TEXELS,
            theme::BLOCK_TEXELS,
            TextureUsage::Color,
            sampler,
        );
    }
}

pub fn load_town_textures(load: &mut TextureLoad<'_>, rgba: &[u8], width: u32) {
    let sampler = SamplerSettings {
        wrap_u: SamplerWrap::Repeat,
        wrap_v: SamplerWrap::Repeat,
        mag_filter: SamplerFilter::Nearest,
        min_filter: SamplerFilter::Nearest,
        mipmap_filter: SamplerFilter::Nearest,
    };
    for (name, tile) in TOWN_SURFACES {
        queue_decoded_texture(
            load,
            name.to_string(),
            tile_pixels(rgba, width, theme::TILE_TEXELS, theme::TOWN_COLUMNS, tile),
            theme::BLOCK_TEXELS,
            theme::BLOCK_TEXELS,
            TextureUsage::Color,
            sampler,
        );
    }
}

const TOWN_SURFACES: [(&str, u32); 10] = [
    (theme::PLANK_TEXTURE, theme::WALL_PLANK),
    (theme::TOWN_STONE_TEXTURE, theme::WALL_STONE),
    (theme::KEEP_TEXTURE, theme::WALL_KEEP),
    (theme::RED_ROOF_TEXTURE, theme::ROOF_RED),
    (theme::BLUE_ROOF_TEXTURE, theme::ROOF_BLUE),
    (theme::WOOD_DOOR_TEXTURE, theme::DOOR_WOOD),
    (theme::GREY_DOOR_TEXTURE, theme::DOOR_GREY),
    (theme::KEEP_DOOR_TEXTURE, theme::DOOR_KEEP),
    (theme::WOOD_WINDOW_TEXTURE, theme::WINDOW_WOOD),
    (theme::GREY_WINDOW_TEXTURE, theme::WINDOW_GREY),
];

const TOWN_MATERIALS: [(&str, &str); 10] = [
    (theme::PLANK_MATERIAL, theme::PLANK_TEXTURE),
    (theme::TOWN_STONE_MATERIAL, theme::TOWN_STONE_TEXTURE),
    (theme::KEEP_MATERIAL, theme::KEEP_TEXTURE),
    (theme::RED_ROOF_MATERIAL, theme::RED_ROOF_TEXTURE),
    (theme::BLUE_ROOF_MATERIAL, theme::BLUE_ROOF_TEXTURE),
    (theme::WOOD_DOOR_MATERIAL, theme::WOOD_DOOR_TEXTURE),
    (theme::GREY_DOOR_MATERIAL, theme::GREY_DOOR_TEXTURE),
    (theme::KEEP_DOOR_MATERIAL, theme::KEEP_DOOR_TEXTURE),
    (theme::WOOD_WINDOW_MATERIAL, theme::WOOD_WINDOW_TEXTURE),
    (theme::GREY_WINDOW_MATERIAL, theme::GREY_WINDOW_TEXTURE),
];

pub fn insert_materials(registry: &mut MaterialRegistry) {
    for (name, texture) in TOWN_MATERIALS {
        insert_material(
            registry,
            name,
            Material {
                base_color: [1.0, 1.0, 1.0, 1.0],
                base_texture: Some(texture.to_string()),
                roughness: 0.9,
                metallic: 0.0,
                ..Default::default()
            },
        );
    }
    for (name, texture) in [
        (theme::GRASS_MATERIAL, theme::GRASS_TEXTURE),
        (theme::DIRT_MATERIAL, theme::DIRT_TEXTURE),
        (theme::SAND_MATERIAL, theme::SAND_TEXTURE),
        (theme::STONE_MATERIAL, theme::STONE_TEXTURE),
    ] {
        insert_material(
            registry,
            name,
            Material {
                base_color: [1.0, 1.0, 1.0, 1.0],
                base_texture: Some(texture.to_string()),
                roughness: 0.95,
                metallic: 0.0,
                ..Default::default()
            },
        );
    }
    insert_material(
        registry,
        theme::WATER_MATERIAL,
        Material {
            base_color: [0.36, 0.74, 0.96, 0.72],
            alpha_mode: AlphaMode::Blend,
            roughness: 0.12,
            metallic: 0.0,
            ..Default::default()
        },
    );
    insert_material(
        registry,
        theme::SEA_MATERIAL,
        Material {
            base_color: [0.3, 0.66, 0.9, 1.0],
            roughness: 0.35,
            metallic: 0.0,
            ..Default::default()
        },
    );
    insert_material(
        registry,
        theme::POST_MATERIAL,
        Material {
            base_color: [0.32, 0.22, 0.14, 1.0],
            roughness: 0.9,
            metallic: 0.0,
            ..Default::default()
        },
    );
    insert_material(
        registry,
        theme::LAMP_MATERIAL,
        Material {
            base_color: [1.0, 0.8, 0.5, 1.0],
            emissive_factor: [5.0, 3.2, 1.4],
            roughness: 0.4,
            metallic: 0.0,
            ..Default::default()
        },
    );
}

fn speckled(
    base: [u8; 4],
    dark: [u8; 4],
    light: [u8; 4],
    seed: u32,
    dark_share: u32,
    light_share: u32,
) -> Vec<u8> {
    let mut rng = Rng::with_seed(seed as u64);
    let texels = (theme::BLOCK_TEXELS * theme::BLOCK_TEXELS) as usize;
    let mut pixels = Vec::with_capacity(texels * 4);
    for _ in 0..texels {
        let roll = rng.u32(..) % 100;
        let color = if roll < dark_share {
            dark
        } else if roll < dark_share + light_share {
            light
        } else {
            base
        };
        pixels.extend_from_slice(&color);
    }
    pixels
}

pub fn shadow_blob() -> Vec<u8> {
    let side = theme::TILE_TEXELS;
    let middle = (side as f32 - 1.0) / 2.0;
    let mut pixels = Vec::with_capacity((side * side * 4) as usize);
    for y in 0..side {
        for x in 0..side {
            let distance = ((x as f32 - middle).powi(2) + (y as f32 - middle).powi(2)).sqrt();
            let alpha = ((1.0 - distance / middle) * 3.0).clamp(0.0, 1.0);
            pixels.extend_from_slice(&[255, 255, 255, (alpha * 255.0) as u8]);
        }
    }
    pixels
}

pub fn cell_square() -> Vec<u8> {
    let side = theme::TILE_TEXELS;
    let mut pixels = Vec::with_capacity((side * side * 4) as usize);
    for y in 0..side {
        for x in 0..side {
            let edge = x == 0 || y == 0 || x == side - 1 || y == side - 1;
            let alpha = if edge { 255 } else { 120 };
            pixels.extend_from_slice(&[255, 255, 255, alpha]);
        }
    }
    pixels
}

pub fn pack_the_sheet(
    atlases: &mut SpriteAtlases,
    battle: (&[u8], u32, u32),
    town: (&[u8], u32, u32),
) -> Sheet {
    sprite_atlas_create(
        atlases,
        theme::ATLAS,
        theme::ATLAS_SIZE,
        SpriteFilter::Nearest,
    );

    let (rgba, width, height) = battle;
    let (town_rgba, town_width, town_height) = town;
    let mut sheet_rows = [TileTrim::default(); theme::SHEET_TILES];
    let mut town_rows = [TileTrim::default(); theme::TOWN_TILES];
    tile_trim(
        rgba,
        width,
        theme::TILE_TEXELS,
        theme::SHEET_COLUMNS,
        &mut sheet_rows,
    );
    tile_trim(
        town_rgba,
        town_width,
        theme::TILE_TEXELS,
        theme::TOWN_COLUMNS,
        &mut town_rows,
    );

    let tiles = sprite_atlas_insert(atlases, theme::ATLAS, theme::SHEET, rgba, width, height)
        .expect("the sheet fits the atlas");
    let town = sprite_atlas_insert(
        atlases,
        theme::ATLAS,
        theme::TOWN,
        town_rgba,
        town_width,
        town_height,
    )
    .expect("the town sheet fits the atlas");

    let shadow = sprite_atlas_insert(
        atlases,
        theme::ATLAS,
        theme::SHADOW,
        &shadow_blob(),
        theme::TILE_TEXELS,
        theme::TILE_TEXELS,
    )
    .expect("the shadow blob fits the atlas");

    let cell = sprite_atlas_insert(
        atlases,
        theme::ATLAS,
        theme::CELL,
        &cell_square(),
        theme::TILE_TEXELS,
        theme::TILE_TEXELS,
    )
    .expect("the cell square fits the atlas");

    Sheet {
        tiles,
        town,
        shadow,
        cell,
        trim: sheet_rows,
        town_trim: town_rows,
    }
}
