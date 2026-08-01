use crate::data::{CUSTOM_ICONS, palette};
use crate::resources::Sheet;
use crate::theme;
use nightshade_graphics::commands::loading::queue_decoded_texture;
use nightshade_graphics::resources::loading::TextureLoad;
use nightshade_render::commands::material_registry::insert_material;
use nightshade_render::data::decoded_image::decode_to_rgba8;
use nightshade_render::data::material::{AlphaMode, Material};
use nightshade_render::data::texture_data::{
    SamplerFilter, SamplerSettings, SamplerWrap, TextureUsage,
};
use nightshade_render::resources::material_registry::MaterialRegistry;
use nightshade_sprite::commands::sheet::tile_trim;
use nightshade_sprite::prelude::*;
use nightshade_texture::prelude::*;

pub fn load_block_textures(load: &mut TextureLoad<'_>) {
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
            speckled(
                theme::BLOCK_TEXELS,
                theme::GRASS_COLOR,
                theme::GRASS_DARK,
                theme::GRASS_LIGHT,
                11,
                22,
                12,
            ),
        ),
        (
            theme::DIRT_TEXTURE,
            speckled(
                theme::BLOCK_TEXELS,
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
                theme::BLOCK_TEXELS,
                theme::SAND_COLOR,
                theme::SAND_DARK,
                theme::SAND_LIGHT,
                37,
                14,
                14,
            ),
        ),
        (
            theme::PLASTER_TEXTURE,
            speckled(
                theme::BLOCK_TEXELS,
                theme::PLASTER_COLOR,
                theme::PLASTER_DARK,
                theme::PLASTER_LIGHT,
                41,
                8,
                10,
            ),
        ),
        (
            theme::STONE_TEXTURE,
            cobbles(
                theme::BLOCK_TEXELS,
                theme::STONE_COLOR,
                theme::STONE_ALT,
                theme::STONE_MORTAR,
                53,
            ),
        ),
        (
            theme::TILLED_TEXTURE,
            furrows(
                theme::BLOCK_TEXELS,
                theme::TILLED_COLOR,
                theme::TILLED_FURROW,
                71,
            ),
        ),
        (
            theme::WET_TILLED_TEXTURE,
            furrows(
                theme::BLOCK_TEXELS,
                theme::WET_TILLED_COLOR,
                theme::WET_TILLED_FURROW,
                73,
            ),
        ),
        (
            theme::PLANK_TEXTURE,
            planks(
                theme::BLOCK_TEXELS,
                theme::PLANK_COLOR,
                theme::PLANK_DARK,
                theme::PLANK_SEAM,
                83,
            ),
        ),
        (
            theme::RED_PLANK_TEXTURE,
            planks(
                theme::BLOCK_TEXELS,
                theme::RED_PLANK_COLOR,
                theme::RED_PLANK_DARK,
                theme::RED_PLANK_SEAM,
                97,
            ),
        ),
        (
            theme::STONE_WALL_TEXTURE,
            stone_wall(
                theme::BLOCK_TEXELS,
                theme::WALL_STONE_COLOR,
                theme::WALL_STONE_ALT,
                theme::WALL_STONE_MORTAR,
                59,
            ),
        ),
        (
            theme::ROOF_RED_TEXTURE,
            shingles(
                theme::BLOCK_TEXELS,
                theme::ROOF_RED_COLOR,
                theme::ROOF_RED_DARK,
            ),
        ),
        (
            theme::ROOF_DARK_TEXTURE,
            shingles(
                theme::BLOCK_TEXELS,
                theme::ROOF_DARK_COLOR,
                theme::ROOF_DARK_DARK,
            ),
        ),
        (
            theme::ROOF_GREEN_TEXTURE,
            shingles(
                theme::BLOCK_TEXELS,
                theme::ROOF_GREEN_COLOR,
                theme::ROOF_GREEN_DARK,
            ),
        ),
        (
            theme::DOOR_TEXTURE,
            door(
                theme::BLOCK_TEXELS,
                theme::DOOR_COLOR,
                theme::DOOR_DARK,
                theme::DOOR_KNOB,
            ),
        ),
        (
            theme::WINDOW_TEXTURE,
            window(
                theme::BLOCK_TEXELS,
                theme::WINDOW_FRAME,
                theme::WINDOW_GLASS,
                theme::WINDOW_GLASS_LIGHT,
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

pub fn insert_materials(registry: &mut MaterialRegistry) {
    let walls = theme::DUNGEON_WALL_SURFACES
        .into_iter()
        .chain([theme::DUNGEON_CAP_SURFACE])
        .map(|(name, _)| (name, name, theme::HOLLOW_WALL_SHADE));
    let floors = theme::DUNGEON_FLOOR_SURFACES
        .into_iter()
        .map(|(name, _)| (name, name, theme::HOLLOW_FLOOR_SHADE));
    for (name, texture, shade) in walls.chain(floors).collect::<Vec<_>>() {
        insert_material(
            registry,
            name,
            Material {
                base_color: shade,
                base_texture: Some(texture.to_string()),
                roughness: 1.0,
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
        (theme::TILLED_MATERIAL, theme::TILLED_TEXTURE),
        (theme::WET_TILLED_MATERIAL, theme::WET_TILLED_TEXTURE),
        (theme::PLANK_MATERIAL, theme::PLANK_TEXTURE),
        (theme::RED_PLANK_MATERIAL, theme::RED_PLANK_TEXTURE),
        (theme::PLASTER_MATERIAL, theme::PLASTER_TEXTURE),
        (theme::STONE_WALL_MATERIAL, theme::STONE_WALL_TEXTURE),
        (theme::ROOF_RED_MATERIAL, theme::ROOF_RED_TEXTURE),
        (theme::ROOF_DARK_MATERIAL, theme::ROOF_DARK_TEXTURE),
        (theme::ROOF_GREEN_MATERIAL, theme::ROOF_GREEN_TEXTURE),
        (theme::DOOR_MATERIAL, theme::DOOR_TEXTURE),
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
        theme::WINDOW_MATERIAL,
        Material {
            base_color: [1.0, 1.0, 1.0, 1.0],
            base_texture: Some(theme::WINDOW_TEXTURE.to_string()),
            roughness: 0.3,
            metallic: 0.0,
            ..Default::default()
        },
    );
    for (name, color) in [
        (theme::TIMBER_MATERIAL, theme::TIMBER_COLOR),
        (theme::POST_MATERIAL, [0.32, 0.22, 0.14, 1.0]),
        (theme::IRON_MATERIAL, [0.2, 0.2, 0.22, 1.0]),
        (theme::DUNGEON_ROCK_MATERIAL, [0.015, 0.014, 0.02, 1.0]),
    ] {
        insert_material(
            registry,
            name,
            Material {
                base_color: color,
                roughness: 0.9,
                metallic: 0.0,
                ..Default::default()
            },
        );
    }
    insert_material(
        registry,
        theme::WATER_MATERIAL,
        Material {
            base_color: [0.22, 0.48, 0.82, 0.62],
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
            base_color: [0.14, 0.3, 0.48, 1.0],
            roughness: 0.35,
            metallic: 0.0,
            ..Default::default()
        },
    );
    insert_material(
        registry,
        theme::LAMP_MATERIAL,
        Material {
            base_color: [1.0, 0.8, 0.5, 1.0],
            emissive_factor: [1.9, 1.25, 0.55],
            roughness: 0.4,
            metallic: 0.0,
            ..Default::default()
        },
    );
    insert_material(
        registry,
        theme::FORGE_MATERIAL,
        Material {
            base_color: [1.0, 0.5, 0.2, 1.0],
            emissive_factor: theme::FORGE_GLOW,
            roughness: 0.6,
            metallic: 0.0,
            ..Default::default()
        },
    );
}

pub fn load_dungeon_textures(load: &mut TextureLoad<'_>) {
    let sampler = SamplerSettings {
        wrap_u: SamplerWrap::Repeat,
        wrap_v: SamplerWrap::Repeat,
        mag_filter: SamplerFilter::Nearest,
        min_filter: SamplerFilter::Nearest,
        mipmap_filter: SamplerFilter::Nearest,
    };
    let (rgba, width, _) = decode_dungeon_sheet();
    let surfaces: Vec<(&str, u32)> = theme::DUNGEON_WALL_SURFACES
        .into_iter()
        .chain(theme::DUNGEON_FLOOR_SURFACES)
        .chain([theme::DUNGEON_CAP_SURFACE])
        .collect();
    for (name, tile) in surfaces {
        queue_decoded_texture(
            load,
            name.to_string(),
            tile_pixels(&rgba, width, theme::TILE_TEXELS, theme::SHEET_COLUMNS, tile),
            theme::TILE_TEXELS,
            theme::TILE_TEXELS,
            TextureUsage::Color,
            sampler,
        );
    }
}

pub fn decode_dungeon_sheet() -> (Vec<u8>, u32, u32) {
    let decoded =
        decode_to_rgba8(theme::DUNGEON_SHEET_BYTES).expect("the tiny dungeon sheet decodes");
    let (width, height) = (decoded.width, decoded.height);
    (decoded.rgba, width, height)
}

pub fn decode_sheet() -> (Vec<u8>, u32, u32) {
    let decoded = decode_to_rgba8(theme::SHEET_BYTES).expect("the tiny farm sheet decodes");
    let (width, height) = (decoded.width, decoded.height);
    (decoded.rgba, width, height)
}

pub fn pack_the_sheet(atlases: &mut SpriteAtlases) -> Sheet {
    sprite_atlas_create(
        atlases,
        theme::ATLAS,
        theme::ATLAS_SIZE,
        SpriteFilter::Nearest,
    );
    let (rgba, width, height) = decode_sheet();
    let mut trim = [TileTrim::default(); theme::SHEET_TILES];
    tile_trim(
        &rgba,
        width,
        theme::TILE_TEXELS,
        theme::SHEET_COLUMNS,
        &mut trim,
    );
    let tiles = sprite_atlas_insert(atlases, theme::ATLAS, theme::SHEET, &rgba, width, height)
        .expect("the sheet fits the atlas");
    let shadow = sprite_atlas_insert(
        atlases,
        theme::ATLAS,
        theme::SHADOW,
        &radial_blob(theme::TILE_TEXELS, [255, 255, 255], 3.0),
        theme::TILE_TEXELS,
        theme::TILE_TEXELS,
    )
    .expect("the shadow blob fits the atlas");
    let (dungeon_rgba, dungeon_width, dungeon_height) = decode_dungeon_sheet();
    let mut dungeon_trim = [TileTrim::default(); theme::SHEET_TILES];
    tile_trim(
        &dungeon_rgba,
        dungeon_width,
        theme::TILE_TEXELS,
        theme::SHEET_COLUMNS,
        &mut dungeon_trim,
    );
    let dungeon = sprite_atlas_insert(
        atlases,
        theme::ATLAS,
        theme::DUNGEON_SHEET,
        &dungeon_rgba,
        dungeon_width,
        dungeon_height,
    )
    .expect("the dungeon sheet fits the atlas");
    Sheet {
        tiles,
        shadow,
        trim,
        dungeon,
        dungeon_trim,
    }
}

pub fn icon_page() -> Vec<u8> {
    let page = theme::ICON_PAGE;
    let scale = theme::ICON_SCALE;
    let mut pixels = vec![0u8; (page * page * 4) as usize];
    let (sheet, width, height) = decode_sheet();
    let mut put = |x: u32, y: u32, color: [u8; 4]| {
        if x < page && y < page {
            let at = ((y * page + x) * 4) as usize;
            pixels[at..at + 4].copy_from_slice(&color);
        }
    };
    for y in 0..height {
        for x in 0..width {
            let at = ((y * width + x) * 4) as usize;
            let color = [sheet[at], sheet[at + 1], sheet[at + 2], sheet[at + 3]];
            for dy in 0..scale {
                for dx in 0..scale {
                    put(x * scale + dx, y * scale + dy, color);
                }
            }
        }
    }
    for (slot, icon) in CUSTOM_ICONS.iter().enumerate() {
        let left = slot as u32 * theme::ICON_CELL;
        let top = theme::CUSTOM_ICON_ROW * theme::ICON_CELL;
        for (y, row) in icon.iter().enumerate() {
            for (x, character) in row.chars().enumerate() {
                let Some(color) = palette(character) else {
                    continue;
                };
                for dy in 0..scale {
                    for dx in 0..scale {
                        put(
                            left + x as u32 * scale + dx,
                            top + y as u32 * scale + dy,
                            color,
                        );
                    }
                }
            }
        }
    }
    pixels
}
