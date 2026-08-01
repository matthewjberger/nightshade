use crate::contract::{Blocks, ChunkKey, Face, SlabKey};
use crate::data;
use crate::queries::{
    cave_cell, column_face_stacks, column_stacks, opening_bit, slab_chunks, slab_of_chunk,
    slab_of_column, standing_chunk,
};
use crate::resources::{ChunkStage, Ground, Meshes, Sight, VoxelSettings, VoxelViewer, VoxelWorld};
use nightshade::prelude::{Commands, Res, ResMut};
use nightshade_platform::contract::time::Time;
use nightshade_scene::contract::primitives::Visibility;
use nightshade_worker::prelude::*;
use std::collections::{HashSet, VecDeque};

pub fn see_the_slabs(
    time: Res<Time>,
    settings: Res<VoxelSettings>,
    mut world: ResMut<VoxelWorld>,
    mut sight: ResMut<Sight>,
    mut masons: ResMut<Workers<Ground>>,
    mut viewer: ResMut<VoxelViewer>,
    meshes: Res<Meshes>,
    mut commands: Commands,
) {
    let center = standing_chunk(viewer.eye / settings.block_size);
    let moved = sight.center != Some(center) || sight.whole != viewer.whole;
    sight.wait -= time.delta_time;
    if !moved && (sight.stamp == world.sight || sight.wait > 0.0) {
        return;
    }
    sight.center = Some(center);
    sight.whole = viewer.whole;
    sight.stamp = world.sight;
    sight.wait = data::SIGHT_SECONDS;

    let shown = if viewer.whole {
        whole(&world)
    } else {
        reachable(&world, &mut sight, center, settings.view_radius)
    };
    let world = &mut *world;
    for key in shown.difference(&viewer.shown) {
        show_the_slab(&world.blocks, &meshes, &mut commands, *key, true);
        let unmeshed = slab_chunks(*key).any(|holder| {
            world
                .chunks
                .get(&holder)
                .is_some_and(|chunk| chunk.stage == ChunkStage::Decorated)
        });
        if unmeshed {
            world.dirty.insert(*key);
        }
    }
    for key in viewer.shown.difference(&shown) {
        show_the_slab(&world.blocks, &meshes, &mut commands, *key, false);
        if workers_cancel(&mut masons, *key) {
            world.dirty.insert(*key);
        }
    }
    viewer.shown = shown;
}

fn show_the_slab(
    blocks: &Blocks,
    meshes: &Meshes,
    commands: &mut Commands,
    key: SlabKey,
    visible: bool,
) {
    for layer in blocks.layers() {
        if let Some(entity) = meshes.entities.get(&(key, layer)) {
            commands.set(*entity, Visibility { visible });
        }
    }
}

fn whole(world: &VoxelWorld) -> HashSet<SlabKey> {
    let mut shown = HashSet::new();
    for column in world.tops.keys() {
        for stack in column_face_stacks(world, *column) {
            shown.insert(slab_of_column(*column, stack));
        }
    }
    shown
}

fn reachable(
    world: &VoxelWorld,
    sight: &mut Sight,
    center: ChunkKey,
    radius: i32,
) -> HashSet<SlabKey> {
    let mut shown = HashSet::new();
    let stacks = column_stacks();
    for (column, top) in &world.tops {
        if (column[0] - center[0]).abs() > radius || (column[1] - center[2]).abs() > radius {
            continue;
        }
        let mut low = *top;
        for step in [[-1, 0], [1, 0], [0, -1], [0, 1]] {
            if let Some(near) = world.tops.get(&[column[0] + step[0], column[1] + step[1]]) {
                low = low.min(*near);
            }
        }
        let lowest = low.div_euclid(data::SLAB_HIGH).max(stacks.start);
        let highest = (top.div_euclid(data::SLAB_HIGH) + 1).min(stacks.end - 1);
        for stack in lowest..=highest {
            shown.insert(slab_of_column(*column, stack));
        }
    }
    sight.entered.clear();
    sight.entered.resize(data::CAVE_CELLS, 0);
    shown.insert(slab_of_chunk(center));
    let mut queue = VecDeque::new();
    if let Some(cell) = cave_cell(center, center) {
        sight.entered[cell] = u8::MAX;
    }
    queue.push_back((center, None));
    while let Some((key, entry)) = queue.pop_front() {
        let openings = world
            .chunks
            .get(&key)
            .map(|chunk| chunk.openings)
            .unwrap_or(0);
        for (exit, face) in Face::ALL.iter().enumerate() {
            if let Some(entry) = entry
                && openings & opening_bit(entry, exit) == 0
            {
                continue;
            }
            let step = face.normal();
            let next = [key[0] + step[0], key[1] + step[1], key[2] + step[2]];
            let Some(cell) = cave_cell(next, center) else {
                continue;
            };
            if !world.chunks.contains_key(&next) {
                continue;
            }
            shown.insert(slab_of_chunk(next));
            let door = 1u8 << (exit ^ 1);
            if sight.entered[cell] & door != 0 {
                continue;
            }
            sight.entered[cell] |= door;
            queue.push_back((next, Some(exit ^ 1)));
        }
    }
    shown
}
