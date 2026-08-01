use crate::data;
use crate::resources::Sheet;
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
            data::GRASS_TEXTURE,
            speckled(
                data::BLOCK_TEXELS,
                data::GRASS_COLOR,
                data::GRASS_DARK,
                data::GRASS_LIGHT,
                11,
                24,
                14,
            ),
        ),
        (
            data::SAND_TEXTURE,
            speckled(
                data::BLOCK_TEXELS,
                data::SAND_COLOR,
                data::SAND_DARK,
                data::SAND_LIGHT,
                37,
                16,
                14,
            ),
        ),
        (
            data::DIRT_TEXTURE,
            speckled(
                data::BLOCK_TEXELS,
                data::DIRT_COLOR,
                data::DIRT_DARK,
                data::DIRT_LIGHT,
                23,
                20,
                14,
            ),
        ),
        (
            data::PATH_TEXTURE,
            speckled(
                data::BLOCK_TEXELS,
                data::PATH_COLOR,
                data::PATH_DARK,
                data::PATH_LIGHT,
                53,
                22,
                18,
            ),
        ),
        (
            data::PLASTER_TEXTURE,
            speckled(
                data::BLOCK_TEXELS,
                data::PLASTER_COLOR,
                data::PLASTER_DARK,
                data::PLASTER_LIGHT,
                41,
                10,
                12,
            ),
        ),
        (
            data::PLANK_TEXTURE,
            planks(
                data::BLOCK_TEXELS,
                data::PLANK_COLOR,
                data::PLANK_DARK,
                data::PLANK_SEAM,
                83,
            ),
        ),
        (
            data::ROOF_RED_TEXTURE,
            shingles(
                data::BLOCK_TEXELS,
                data::ROOF_RED_COLOR,
                data::ROOF_RED_DARK,
            ),
        ),
        (
            data::ROOF_BLUE_TEXTURE,
            shingles(
                data::BLOCK_TEXELS,
                data::ROOF_BLUE_COLOR,
                data::ROOF_BLUE_DARK,
            ),
        ),
        (
            data::ROOF_GREEN_TEXTURE,
            shingles(
                data::BLOCK_TEXELS,
                data::ROOF_GREEN_COLOR,
                data::ROOF_GREEN_DARK,
            ),
        ),
        (
            data::DOOR_TEXTURE,
            door(
                data::BLOCK_TEXELS,
                data::DOOR_COLOR,
                data::DOOR_DARK,
                data::DOOR_KNOB,
            ),
        ),
        (
            data::WINDOW_TEXTURE,
            window(
                data::BLOCK_TEXELS,
                data::WINDOW_FRAME,
                data::WINDOW_GLASS,
                data::WINDOW_GLASS_LIGHT,
            ),
        ),
        (data::AWNING_TEXTURE, awning()),
    ];
    for (name, rgba) in textures {
        queue_decoded_texture(
            load,
            name.to_string(),
            rgba,
            data::BLOCK_TEXELS,
            data::BLOCK_TEXELS,
            TextureUsage::Color,
            sampler,
        );
    }
}

pub fn insert_materials(registry: &mut MaterialRegistry) {
    for (name, texture) in [
        (data::GRASS_MATERIAL, data::GRASS_TEXTURE),
        (data::SAND_MATERIAL, data::SAND_TEXTURE),
        (data::DIRT_MATERIAL, data::DIRT_TEXTURE),
        (data::PATH_MATERIAL, data::PATH_TEXTURE),
        (data::PLANK_MATERIAL, data::PLANK_TEXTURE),
        (data::PLASTER_MATERIAL, data::PLASTER_TEXTURE),
        (data::ROOF_RED_MATERIAL, data::ROOF_RED_TEXTURE),
        (data::ROOF_BLUE_MATERIAL, data::ROOF_BLUE_TEXTURE),
        (data::ROOF_GREEN_MATERIAL, data::ROOF_GREEN_TEXTURE),
        (data::DOOR_MATERIAL, data::DOOR_TEXTURE),
        (data::AWNING_MATERIAL, data::AWNING_TEXTURE),
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
        data::WINDOW_MATERIAL,
        Material {
            base_color: [1.0, 1.0, 1.0, 1.0],
            base_texture: Some(data::WINDOW_TEXTURE.to_string()),
            roughness: 0.3,
            metallic: 0.0,
            ..Default::default()
        },
    );
    insert_material(
        registry,
        data::TIMBER_MATERIAL,
        Material {
            base_color: data::TIMBER_COLOR,
            roughness: 0.9,
            metallic: 0.0,
            ..Default::default()
        },
    );
    insert_material(
        registry,
        data::WATER_MATERIAL,
        Material {
            base_color: data::WATER_COLOR,
            alpha_mode: AlphaMode::Blend,
            roughness: 0.1,
            metallic: 0.0,
            ..Default::default()
        },
    );
    insert_material(
        registry,
        data::LAMP_MATERIAL,
        Material {
            base_color: data::LAMP_COLOR,
            emissive_factor: data::LAMP_GLOW,
            roughness: 0.4,
            metallic: 0.0,
            ..Default::default()
        },
    );
}

pub fn pack_the_sheet(atlases: &mut SpriteAtlases) -> Sheet {
    sprite_atlas_create(
        atlases,
        data::ATLAS,
        data::ATLAS_SIZE,
        SpriteFilter::Nearest,
    );
    let (rgba, width, height) = decode_sheet();
    let mut trim = [TileTrim::default(); data::SHEET_TILES];
    tile_trim(
        &rgba,
        width,
        data::TILE_TEXELS,
        data::SHEET_COLUMNS,
        &mut trim,
    );
    let tiles = sprite_atlas_insert(atlases, data::ATLAS, data::SHEET, &rgba, width, height)
        .expect("the sheet fits the atlas");
    let shadow = sprite_atlas_insert(
        atlases,
        data::ATLAS,
        data::SHADOW,
        &radial_blob(data::TILE_TEXELS, [255, 255, 255], 3.0),
        data::TILE_TEXELS,
        data::TILE_TEXELS,
    )
    .expect("the shadow blob fits the atlas");
    let mut villagers = [SpriteRegion::default(); data::NEIGHBORS.len()];
    for (index, neighbor) in data::NEIGHBORS.iter().enumerate() {
        let shape = match neighbor.shape {
            0 => data::CAT_SHAPE,
            1 => data::DOG_SHAPE,
            2 => data::BIRD_SHAPE,
            _ => data::BEAR_SHAPE,
        };
        villagers[index] = sprite_atlas_insert(
            atlases,
            data::ATLAS,
            format!("villager_{index}"),
            &paint_villager(&shape, neighbor),
            data::VILLAGER_TEXELS,
            data::VILLAGER_TEXELS,
        )
        .expect("the villager fits the atlas");
    }

    let shell = sprite_atlas_insert(
        atlases,
        data::ATLAS,
        "shell",
        &paint_shape(&data::SHELL_SHAPE, data::SHELL_COAT, data::SHELL_LIGHT),
        data::VILLAGER_TEXELS,
        data::VILLAGER_TEXELS,
    )
    .expect("the shell fits the atlas");
    let fish = sprite_atlas_insert(
        atlases,
        data::ATLAS,
        "fish",
        &paint_shape(&data::FISH_SHAPE, data::FISH_COAT, data::FISH_LIGHT),
        data::VILLAGER_TEXELS,
        data::VILLAGER_TEXELS,
    )
    .expect("the fish fits the atlas");

    Sheet {
        tiles,
        shadow,
        trim,
        villagers,
        shell,
        fish,
    }
}

fn paint_shape(shape: &[&str; 16], coat: [u8; 4], light: [u8; 4]) -> Vec<u8> {
    let mut pixels =
        Vec::with_capacity((data::VILLAGER_TEXELS * data::VILLAGER_TEXELS * 4) as usize);
    for line in shape {
        for mark in line.chars() {
            let color = match mark {
                'o' => data::VILLAGER_OUTLINE,
                'F' => coat,
                'f' => light,
                'e' => data::VILLAGER_EYE,
                'w' => data::VILLAGER_WHITE,
                _ => [0, 0, 0, 0],
            };
            pixels.extend_from_slice(&color);
        }
    }
    pixels
}

fn paint_villager(shape: &[&str; 16], neighbor: &data::Neighbor) -> Vec<u8> {
    let mut pixels =
        Vec::with_capacity((data::VILLAGER_TEXELS * data::VILLAGER_TEXELS * 4) as usize);
    for line in shape {
        for mark in line.chars() {
            let color = match mark {
                'o' => data::VILLAGER_OUTLINE,
                'F' => neighbor.coat,
                'f' => neighbor.belly,
                's' => neighbor.shirt,
                'e' => data::VILLAGER_EYE,
                'w' => data::VILLAGER_WHITE,
                'n' => data::VILLAGER_NOSE,
                'b' => data::VILLAGER_BOOT,
                _ => [0, 0, 0, 0],
            };
            pixels.extend_from_slice(&color);
        }
    }
    pixels
}

fn awning() -> Vec<u8> {
    paint(data::BLOCK_TEXELS, |x, _| {
        if (x / 3).is_multiple_of(2) {
            data::AWNING_STRIPE
        } else {
            data::AWNING_COLOR
        }
    })
}

fn decode_sheet() -> (Vec<u8>, u32, u32) {
    let decoded = decode_to_rgba8(data::SHEET_BYTES).expect("the village sheet decodes");
    let (width, height) = (decoded.width, decoded.height);
    (decoded.rgba, width, height)
}
