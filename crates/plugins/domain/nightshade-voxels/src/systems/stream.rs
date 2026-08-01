use crate::commands::{drop_the_mesh, hand_out_the_column, unload_column};
use crate::contract::{ChunkLayer, ColumnKey};
use crate::data;
use crate::queries::{
    column_of, column_settled, column_stacks, slab_columns, slab_mesh_name, slab_of_column,
    view_rank, view_sector,
};
use crate::resources::{ColumnJob, Dig, Meshes, Streaming, VoxelSettings, VoxelViewer, VoxelWorld};
use nalgebra_glm::Vec3;
use nightshade::prelude::{Commands, Res, ResMut};
use nightshade_render::commands::mesh_cache::mesh_cache_release;
use nightshade_render::prelude::MeshCache;
use nightshade_worker::prelude::*;
use std::collections::HashSet;

pub fn stream_the_world<D: Dig>(
    viewer: Res<VoxelViewer>,
    settings: Res<VoxelSettings>,
    mut world: ResMut<VoxelWorld>,
    mut orders: ResMut<D::Orders>,
    mut diggers: ResMut<Workers<ColumnJob<D>>>,
    mut streaming: ResMut<Streaming>,
    mut meshes: ResMut<Meshes>,
    mut mesh_cache: ResMut<MeshCache>,
    mut commands: Commands,
) {
    let eye = viewer.eye / settings.block_size;
    let center = column_of(eye.x.floor() as i32, eye.z.floor() as i32);
    let sector = view_sector(viewer.forward);
    let moved = streaming.center != Some(center);
    let turned = streaming.facing != Some(sector);
    if moved || turned {
        streaming.center = Some(center);
        streaming.facing = Some(sector);
        streaming.pending = wanted_columns(center, viewer.forward, settings.view_radius);
    }

    while workers_in_flight(&diggers) < data::COLUMNS_IN_FLIGHT {
        let Some(column) = streaming.pending.pop() else {
            break;
        };
        if column_settled(&world, column) || workers_holds(&diggers, column) {
            continue;
        }
        let rank = view_rank(
            column[0] - center[0],
            column[1] - center[1],
            viewer.forward,
            data::VIEW_BEHIND,
        );
        hand_out_the_column::<D>(&world, &mut orders, &mut diggers, column, rank);
    }

    let keep = settings.keep_radius;
    if moved {
        let gone: HashSet<ColumnKey> = world
            .surface
            .keys()
            .copied()
            .filter(|column| {
                (column[0] - center[0]).abs() > keep || (column[1] - center[1]).abs() > keep
            })
            .collect();
        streaming.leaving.clear();
        streaming.leaving.extend(gone);
    }
    let taken = data::UNLOAD_STEP.min(streaming.leaving.len());
    let leaving: Vec<ColumnKey> = streaming.leaving.drain(..taken).collect();
    let layers: Vec<ChunkLayer> = world.blocks.layers().collect();
    for column in leaving {
        if (column[0] - center[0]).abs() <= keep && (column[1] - center[1]).abs() <= keep {
            continue;
        }
        workers_cancel(&mut diggers, column);
        unload_column(&mut world, column);
        let stack = slab_of_column(column, 0);
        if slab_columns(stack).any(|held| world.surface.contains_key(&held)) {
            continue;
        }
        for stacked in column_stacks() {
            let key = [stack[0], stacked, stack[2]];
            meshes.casting.remove(&key);
            for &layer in &layers {
                mesh_cache_release(&mut mesh_cache, &slab_mesh_name(key, layer));
                if let Some(entity) = meshes.entities.remove(&(key, layer)) {
                    drop_the_mesh(&mut commands, entity);
                }
            }
        }
    }
}

fn wanted_columns(center: ColumnKey, forward: Vec3, radius: i32) -> Vec<ColumnKey> {
    let mut columns = Vec::new();
    for dz in -radius..=radius {
        for dx in -radius..=radius {
            if dx * dx + dz * dz > radius * radius {
                continue;
            }
            columns.push([center[0] + dx, center[1] + dz]);
        }
    }
    columns.sort_by_cached_key(|column| {
        -view_rank(
            column[0] - center[0],
            column[1] - center[1],
            forward,
            data::VIEW_BEHIND,
        )
    });
    columns
}
