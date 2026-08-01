use crate::data;
use crate::resources::{Land, Machine, Script, Store};
use nalgebra_glm::IVec3;
use nightshade::prelude::*;
use nightshade_graphics::commands::loading::queue_decoded_texture;
use nightshade_graphics::resources::loading::TextureLoad;
use nightshade_noise::prelude::layered;
use nightshade_render::commands::material_registry::insert_material;
use nightshade_render::commands::texture_cache::texture_cache_protect;
use nightshade_render::data::material::Material;
use nightshade_render::data::texture_data::{
    SamplerFilter, SamplerSettings, SamplerWrap, TextureUsage,
};
use nightshade_render::prelude::MaterialRegistry;
use nightshade_scene::prelude::GlobalTransform;
use nightshade_voxels::prelude::{
    AIR, BlockLook, Blocks, Dig, Shape, VoxelSettings, VoxelViewer, VoxelWorld, reset_the_world,
    set_block, write_block,
};
use std::sync::Arc;

pub struct Digger;

impl Dig for Digger {
    type Orders = Land;
    type Order = Land;
    type Tools = ();
    type Report = ();

    fn order(orders: &mut Land, _column: [i32; 2]) -> Land {
        *orders
    }

    fn dig(order: Land, _tools: &mut (), world: &mut VoxelWorld, column: [i32; 2]) {
        if !order.on {
            return;
        }
        let side = data::LAND_SIDE;
        let shore = data::LAND_FLOOR + order.shore.round() as i32;
        for step in 0..side * side {
            let across = column[0] * side + step % side;
            let along = column[1] * side + step / side;
            if across.abs() > data::LAND_SPAN || along.abs() > data::LAND_SPAN {
                continue;
            }
            let shape = layered(
                across as f32 * data::LAND_GRAIN,
                along as f32 * data::LAND_GRAIN,
                order.seed,
                4,
                0.5,
            );
            let top = data::LAND_FLOOR + (shape * order.hills).round() as i32;
            for height in data::LAND_FLOOR - data::LAND_DEPTH..=top.max(shore) {
                let block = match height {
                    held if held > top => 4,
                    held if held == top && top >= shore => 3,
                    held if held == top => 4,
                    held if held > top - data::LAND_SKIN => 2,
                    _ => 1,
                };
                write_block(world, IVec3::new(across, height, along), block);
            }
        }
    }

    fn settle(
        _orders: &mut Land,
        _world: &mut VoxelWorld,
        _column: [i32; 2],
        _report: (),
    ) -> Vec<[i32; 2]> {
        Vec::new()
    }
}

pub fn dress(
    mut registry: ResMut<MaterialRegistry>,
    mut world: ResMut<VoxelWorld>,
    mut settings: ResMut<VoxelSettings>,
    mut textures: TextureLoad,
) {
    let side = data::LAND_CELL_SIDE;
    queue_decoded_texture(
        &mut textures,
        data::LAND_TEXTURE.to_string(),
        atlas(),
        side * data::LAND_CELLS[0],
        side * data::LAND_CELLS[1],
        TextureUsage::Color,
        SamplerSettings {
            wrap_u: SamplerWrap::ClampToEdge,
            wrap_v: SamplerWrap::ClampToEdge,
            mag_filter: SamplerFilter::Nearest,
            min_filter: SamplerFilter::Nearest,
            mipmap_filter: SamplerFilter::Nearest,
        },
    );
    texture_cache_protect(textures.cache, data::LAND_TEXTURE.to_string());
    for name in [
        data::LAND_SOLID_MATERIAL,
        data::LAND_CUTOUT_MATERIAL,
        data::LAND_CLEAR_MATERIAL,
    ] {
        insert_material(
            &mut registry,
            name,
            Material {
                base_color: [1.0, 1.0, 1.0, 1.0],
                base_texture: Some(data::LAND_TEXTURE.to_string()),
                atlas_cells: [data::LAND_CELLS[0] as f32, data::LAND_CELLS[1] as f32],
                sky_light: true,
                vertex_occlusion: true,
                roughness: 1.0,
                metallic: 0.0,
                ..Default::default()
            },
        );
    }
    *settings = VoxelSettings {
        solid_material: data::LAND_SOLID_MATERIAL.to_string(),
        cutout_material: data::LAND_CUTOUT_MATERIAL.to_string(),
        transparent_material: data::LAND_CLEAR_MATERIAL.to_string(),
        floor_block: 1,
        view_radius: data::LAND_REACH,
        keep_radius: data::LAND_REACH + 2,
        ..Default::default()
    };
    world.blocks = Arc::new(table());
}

fn table() -> Blocks {
    let mut looks = vec![BlockLook {
        occludes: false,
        solid: false,
        replaceable: true,
        ..Default::default()
    }];
    for cell in 0..data::LAND_KINDS.len() as u32 {
        looks.push(BlockLook {
            top: (cell, 0),
            bottom: (cell, 0),
            side: (cell, 0),
            shape: Shape::Cube,
            occludes: true,
            solid: true,
            ..Default::default()
        });
    }
    Blocks {
        looks,
        atlas_columns: data::LAND_CELLS[0],
        atlas_rows: data::LAND_CELLS[1],
        ..Default::default()
    }
}

fn atlas() -> Vec<u8> {
    let side = data::LAND_CELL_SIDE;
    let wide = side * data::LAND_CELLS[0];
    let tall = side * data::LAND_CELLS[1];
    let tones = [
        [0.44, 0.44, 0.48],
        [0.42, 0.31, 0.22],
        [0.34, 0.58, 0.28],
        [0.78, 0.72, 0.48],
    ];
    let mut pixels = Vec::with_capacity((wide * tall * 4) as usize);
    for row in 0..tall {
        for column in 0..wide {
            let kind = (column / side).min(3) as usize;
            let grain = ((row * 11 + column * 7) % 13) as f32 / 13.0;
            let edge = row % side < 1 || column % side < 1;
            let shade = match edge {
                true => 0.82,
                false => 0.92 + grain * 0.16,
            };
            for part in tones[kind] {
                pixels.push((part * shade * 255.0).clamp(0.0, 255.0) as u8);
            }
            pixels.push(255);
        }
    }
    pixels
}

pub fn settle(
    script: Res<Script>,
    machine: Res<Machine>,
    mut store: ResMut<Store>,
    mut orders: ResMut<Land>,
    mut world: ResMut<VoxelWorld>,
    mut viewer: ResMut<VoxelViewer>,
    mut settings: ResMut<VoxelSettings>,
    mut commands: Commands,
    mut places: Query<(&GlobalTransform,)>,
) {
    let known = &script.plan.known;
    let wanted = Land {
        on: known.land_seed.is_some()
            || known.land_hills.is_some()
            || known.land_shore.is_some()
            || known.land_reach.is_some(),
        seed: known
            .land_seed
            .map(|slot| store.field(slot).value().max(0.0) as u32)
            .unwrap_or(1),
        hills: known
            .land_hills
            .map(|slot| store.field(slot).value())
            .unwrap_or(data::LAND_HILLS),
        shore: known
            .land_shore
            .map(|slot| store.field(slot).value())
            .unwrap_or(data::LAND_SHORE),
    };
    let reach = known
        .land_reach
        .map(|slot| store.field(slot).value())
        .unwrap_or(data::LAND_REACH as f32)
        .clamp(1.0, data::LAND_FARTHEST as f32) as i32;
    if settings.view_radius != reach {
        settings.view_radius = reach;
        settings.keep_radius = reach + 2;
    }
    if !store.dug || *orders != wanted {
        store.dug = true;
        *orders = wanted;
        commands.queue(reset_the_world::<Digger>);
    }
    if !wanted.on {
        return;
    }
    viewer.whole = true;
    if let Some(camera) = store.camera {
        places.for_entity(camera, |(place,)| {
            viewer.eye = place.translation();
            viewer.forward = place.forward_vector();
        });
    }
    let width = store.rows.len() as u32;
    for row in 0..width {
        if !store.living(row) {
            continue;
        }
        let Some(transform) = known.transform.filter(|key| store.holds(*key, row)) else {
            continue;
        };
        let held = store.read(transform, row);
        let spot = IVec3::new(
            held.axis(0).floor() as i32,
            held.axis(1).floor() as i32,
            held.axis(2).floor() as i32,
        );
        let digs = known
            .dig
            .filter(|key| store.holds(*key, row))
            .map(|key| store.read(key, row).value() > 0.5)
            .unwrap_or(false);
        if digs {
            set_block(&mut world, spot, AIR);
        }
        let Some(key) = known.block.filter(|key| store.holds(*key, row)) else {
            continue;
        };
        let name = crate::queries::show(&store, &machine, store.read(key, row));
        let Some(kind) = data::LAND_KINDS.iter().position(|held| *held == name) else {
            continue;
        };
        set_block(&mut world, spot, kind as u8 + 1);
    }
}
