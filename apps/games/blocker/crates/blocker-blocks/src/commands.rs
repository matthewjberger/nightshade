use crate::data::blocks::{BlockKind, Face, Palette, Shape, block_table};
use crate::data::items::{ART_ITEMS, Item};
use crate::queries::{art_mesh, crack_mesh, cube_mesh, extruded_cell_mesh, ripple_pixels};
use crate::queries::{decode_atlas, empty_rows_below};
use crate::resources::Sheet;
use crate::theme;
use nalgebra_glm::IVec3;
use nightshade_graphics::commands::loading::queue_decoded_texture;
use nightshade_graphics::resources::loading::TextureLoad;
use nightshade_render::commands::material_registry::insert_material;
use nightshade_render::commands::mesh_cache::{mesh_cache_insert, mesh_cache_protect};
use nightshade_render::commands::texture_cache::texture_cache_protect;
use nightshade_render::data::decoded_image::decode_to_rgba8;
use nightshade_render::data::material::{AlphaMode, Material, TextureTransform};
use nightshade_render::data::texture_data::{
    SamplerFilter, SamplerSettings, SamplerWrap, TextureUsage,
};
use nightshade_render::prelude::MeshCache;
use nightshade_render::resources::material_registry::MaterialRegistry;
use nightshade_sprite::prelude::*;
use nightshade_voxels::commands::set_block;
use nightshade_voxels::prelude::Blocks;
use nightshade_voxels::queries::block_at;
use nightshade_voxels::resources::{VoxelSettings, VoxelWorld};
use std::sync::Arc;

pub fn register_item_meshes(cache: &mut MeshCache, pixels: &[u8], width: u32) {
    let table = block_table();
    for kind in crate::data::blocks::BLOCKS.map(|row| row.kind) {
        if matches!(kind, BlockKind::Air | BlockKind::Water) {
            continue;
        }
        let name = Item::Block(kind).mesh_name();
        let info = kind.info();
        let mesh = if info.shape == Shape::Cube {
            cube_mesh(
                &table,
                |face| match face {
                    Face::Top => info.top,
                    Face::Bottom => info.bottom,
                    _ => info.side,
                },
                -0.5,
                0.5,
            )
        } else {
            extruded_cell_mesh(pixels, width, kind.info().side)
        };
        mesh_cache_insert(cache, name.clone(), mesh);
        mesh_cache_protect(cache, name);
    }
    for item in ART_ITEMS {
        let Some(art) = item.art() else {
            continue;
        };
        let name = item.mesh_name();
        mesh_cache_insert(cache, name.clone(), art_mesh(art));
        mesh_cache_protect(cache, name);
    }
    mesh_cache_insert(
        cache,
        crate::data::atlas::CRACK_MESH.to_string(),
        crack_mesh(&table, 0),
    );
    mesh_cache_protect(cache, crate::data::atlas::CRACK_MESH.to_string());
}

pub fn update_crack_mesh(cache: &mut MeshCache, blocks: &Blocks, stage: usize) {
    mesh_cache_insert(
        cache,
        crate::data::atlas::CRACK_MESH.to_string(),
        crack_mesh(blocks, stage),
    );
}

pub fn load_ripples(load: &mut TextureLoad<'_>) {
    let sampler = SamplerSettings {
        wrap_u: SamplerWrap::Repeat,
        wrap_v: SamplerWrap::Repeat,
        mag_filter: SamplerFilter::Linear,
        min_filter: SamplerFilter::Linear,
        mipmap_filter: SamplerFilter::Linear,
    };
    queue_decoded_texture(
        load,
        theme::RIPPLE_TEXTURE.to_string(),
        ripple_pixels(),
        theme::RIPPLE_TEXELS,
        theme::RIPPLE_TEXELS,
        TextureUsage::Linear,
        sampler,
    );
    texture_cache_protect(load.cache, theme::RIPPLE_TEXTURE.to_string());
}

pub fn load_atlas(load: &mut TextureLoad<'_>, pixels: Vec<u8>, width: u32, height: u32) {
    let sampler = SamplerSettings {
        wrap_u: SamplerWrap::ClampToEdge,
        wrap_v: SamplerWrap::ClampToEdge,
        mag_filter: SamplerFilter::Nearest,
        min_filter: SamplerFilter::Nearest,
        mipmap_filter: SamplerFilter::Nearest,
    };
    queue_decoded_texture(
        load,
        crate::data::atlas::ATLAS_TEXTURE.to_string(),
        pixels,
        width,
        height,
        TextureUsage::Color,
        sampler,
    );
    texture_cache_protect(load.cache, crate::data::atlas::ATLAS_TEXTURE.to_string());
}

pub fn insert_materials(registry: &mut MaterialRegistry) {
    insert_material(
        registry,
        theme::SOLID_MATERIAL,
        Material {
            base_color: [1.0, 1.0, 1.0, 1.0],
            base_texture: Some(crate::data::atlas::ATLAS_TEXTURE.to_string()),
            atlas_cells: crate::data::atlas::ATLAS_CELLS,
            sky_light: true,
            vertex_occlusion: true,
            roughness: 1.0,
            metallic: 0.0,
            ..Default::default()
        },
    );
    insert_material(
        registry,
        crate::data::atlas::CUTOUT_MATERIAL,
        Material {
            base_color: [1.0, 1.0, 1.0, 1.0],
            base_texture: Some(crate::data::atlas::ATLAS_TEXTURE.to_string()),
            atlas_cells: crate::data::atlas::ATLAS_CELLS,
            sky_light: true,
            vertex_occlusion: true,
            alpha_mode: AlphaMode::Mask,
            alpha_cutoff: 0.15,
            double_sided: false,
            roughness: 1.0,
            metallic: 0.0,
            ..Default::default()
        },
    );
    insert_material(
        registry,
        crate::data::atlas::TRANSPARENT_MATERIAL,
        Material {
            base_color: [1.0, 1.0, 1.0, 1.0],
            base_texture: Some(crate::data::atlas::ATLAS_TEXTURE.to_string()),
            atlas_cells: crate::data::atlas::ATLAS_CELLS,
            sky_light: true,
            vertex_occlusion: true,
            alpha_mode: AlphaMode::Blend,
            double_sided: false,
            roughness: theme::GLASS_ROUGHNESS,
            metallic: 0.0,
            ..Default::default()
        },
    );
    insert_material(
        registry,
        crate::data::atlas::WATER_MATERIAL,
        Material {
            base_color: theme::WATER_COLOR,
            base_texture: Some(crate::data::atlas::ATLAS_TEXTURE.to_string()),
            base_texture_transform: TextureTransform {
                drift: theme::WATER_DRIFT,
                ..Default::default()
            },
            atlas_cells: crate::data::atlas::ATLAS_CELLS,
            normal_texture: Some(theme::RIPPLE_TEXTURE.to_string()),
            normal_texture_transform: TextureTransform {
                scale: [theme::RIPPLE_SCALE, theme::RIPPLE_SCALE],
                drift: theme::RIPPLE_DRIFT,
                ..Default::default()
            },
            normal_scale: theme::RIPPLE_STRENGTH,
            sky_light: true,
            vertex_occlusion: true,
            alpha_mode: AlphaMode::Blend,
            double_sided: true,
            roughness: theme::WATER_ROUGHNESS,
            metallic: 0.0,
            ..Default::default()
        },
    );
    insert_material(
        registry,
        crate::data::atlas::FLAT_MATERIAL,
        Material {
            base_color: [1.0, 1.0, 1.0, 1.0],
            roughness: 0.8,
            metallic: 0.1,
            ..Default::default()
        },
    );
}

pub fn pack_the_sheet(atlases: &mut SpriteAtlases) -> Sheet {
    sprite_atlas_create(
        atlases,
        crate::data::atlas::SPRITE_ATLAS,
        crate::data::atlas::SPRITE_ATLAS_SIZE,
        SpriteFilter::Nearest,
    );
    let decoded = decode_to_rgba8(theme::SHEET_BYTES).expect("the sheet decodes");
    let (width, height) = (decoded.width, decoded.height);
    let empty_rows_below = empty_rows_below(&decoded.rgba, width);
    let tiles = sprite_atlas_insert(
        atlases,
        crate::data::atlas::SPRITE_ATLAS,
        theme::SPRITE_SHEET,
        &decoded.rgba,
        width,
        height,
    )
    .expect("the sheet fits the atlas");
    Sheet {
        tiles,
        empty_rows_below,
    }
}

pub fn swing_the_door(world: &mut VoxelWorld, block: IVec3) {
    let up = IVec3::new(0, 1, 0);
    let kind = BlockKind::from_index(block_at(world, block));
    let Some((facing, open)) = kind.door_facing() else {
        return;
    };
    let lowest = if block_at(world, block - up) == kind.index() {
        block - up
    } else {
        block
    };
    let wanted = if open {
        BlockKind::door_kind(facing, false)
    } else {
        BlockKind::door_kind(facing, true)
    };
    let wanted = if open {
        wanted
    } else {
        let axis = IVec3::new(facing.z.abs(), 0, facing.x.abs());
        let away = if door_beside(world, lowest, axis, facing) {
            -axis
        } else if door_beside(world, lowest, -axis, facing) {
            axis
        } else {
            BlockKind::door_swing(facing, false)
        };
        if BlockKind::door_swing(facing, false) == away {
            wanted
        } else {
            BlockKind::door_mirrored_kind(facing)
        }
    };
    for step in 0..2 {
        let position = lowest + up * step;
        if BlockKind::from_index(block_at(world, position))
            .door_facing()
            .is_none()
        {
            continue;
        }
        set_block(world, position, wanted.index());
    }
}

fn door_beside(world: &VoxelWorld, block: IVec3, step: IVec3, facing: IVec3) -> bool {
    BlockKind::from_index(block_at(world, block + step))
        .door_facing()
        .is_some_and(|(beside, _)| beside == facing)
}

pub fn install_the_blocks(
    cache: &mut MeshCache,
    load: &mut TextureLoad<'_>,
    registry: &mut MaterialRegistry,
    atlases: &mut SpriteAtlases,
    sheet: &mut Sheet,
    settings: &mut VoxelSettings,
    world: &mut VoxelWorld,
) {
    let (pixels, width, height) = decode_atlas(Palette::Kenney);
    register_item_meshes(cache, &pixels, width);
    load_atlas(load, pixels, width, height);
    load_ripples(load);
    insert_materials(registry);
    *sheet = pack_the_sheet(atlases);
    settings.solid_material = theme::SOLID_MATERIAL.to_string();
    settings.cutout_material = crate::data::atlas::CUTOUT_MATERIAL.to_string();
    settings.transparent_material = crate::data::atlas::TRANSPARENT_MATERIAL.to_string();
    settings.floor_block = BlockKind::Bedrock.index();
    world.blocks = Arc::new(block_table());
}
