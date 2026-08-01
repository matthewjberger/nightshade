use crate::contract::{AIR, ColumnKey};
use crate::data;
use crate::queries::{
    block_at, block_index, box_overlaps_block, chunk_of, chunk_openings, chunks_of,
    column_face_stacks, column_of, column_stacks, column_top, in_bounds, is_solid_at,
    measure_surface, slab_mesh_name, slab_of, slab_of_column, surface_index,
};
use crate::resources::{
    ChunkStage, ColumnJob, Dig, Falling, Ground, Meshes, Sight, Streaming, VoxelWorld,
};
use nalgebra_glm::{IVec3, Vec3};
use nightshade::ecs::{Entity, ResourceMap, Storage};
use nightshade::prelude::Commands;
use nightshade_graphics::commands::builtins::despawn_entities_with_cache_cleanup;
use nightshade_render::commands::mesh_cache::mesh_cache_release;
use nightshade_render::commands::mesh_state::mesh_state_mark_entity_removed;
use nightshade_render::prelude::{MeshCache, MeshRenderState};
use nightshade_worker::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;

pub fn write_block(world: &mut VoxelWorld, position: IVec3, block: u8) {
    if !in_bounds(position) {
        return;
    }
    let key = chunk_of(position);
    let index = block_index(position);
    let chunk = world.chunks.entry(key).or_default();
    let previous = chunk.blocks[index];
    if previous == block {
        return;
    }
    Arc::make_mut(&mut chunk.blocks)[index] = block;
    if block == AIR {
        chunk.solid = false;
    }
    raise_the_surface(world, position, block);
    let cell = [position.x, position.y, position.z];
    if world.blocks.look(block).marked {
        world.markers.entry(key).or_default().insert(cell, block);
    } else if world.blocks.look(previous).marked
        && let Some(list) = world.markers.get_mut(&key)
    {
        list.remove(&cell);
    }
}

fn raise_the_surface(world: &mut VoxelWorld, position: IVec3, block: u8) {
    let occludes = world.blocks.occludes(block);
    let column = column_of(position.x, position.z);
    let cell = surface_index(position.x, position.z);
    let Some(heights) = world.surface.get_mut(&column).map(Arc::make_mut) else {
        return;
    };
    let top = heights[cell] as i32;
    if occludes {
        if position.y > top {
            heights[cell] = position.y as i16;
            let raised = column_top(heights);
            world.tops.insert(column, raised);
        }
        return;
    }
    if position.y != top {
        return;
    }
    heights[cell] = data::SURFACE_NONE;
    for y in (data::FLOOR_Y..position.y).rev() {
        let below = IVec3::new(position.x, y, position.z);
        let Some(chunk) = world.chunks.get(&chunk_of(below)) else {
            continue;
        };
        if world.blocks.occludes(chunk.blocks[block_index(below)]) {
            if let Some(heights) = world.surface.get_mut(&column).map(Arc::make_mut) {
                heights[cell] = y as i16;
                let lowered = column_top(heights);
                world.tops.insert(column, lowered);
            }
            return;
        }
    }
}

pub fn set_block(world: &mut VoxelWorld, position: IVec3, block: u8) {
    if !in_bounds(position) {
        return;
    }
    write_block(world, position, block);
    let key = chunk_of(position);
    let index = block_index(position) as u16;
    world.edited.entry(key).or_default().insert(index, block);
    world.edits += 1;
    world.sight += 1;
    world.updates.push(position);
    world.wet.push(position);
    for axis in 0..3 {
        for step in [-1, 1] {
            let mut offset = IVec3::zeros();
            offset[axis] = step;
            world.updates.push(position + offset);
            world.wet.push(position + offset);
        }
    }
    mark_around_dirty(world, position);
}

pub fn mark_around_dirty(world: &mut VoxelWorld, position: IVec3) {
    for x in -1..=1 {
        for y in -1..=1 {
            for z in -1..=1 {
                let neighbor = position + IVec3::new(x, y, z);
                world
                    .dirty
                    .insert(slab_of(neighbor.x, neighbor.y, neighbor.z));
            }
        }
    }
}

pub fn mark_column_dirty(world: &mut VoxelWorld, column: ColumnKey) {
    for dz in -1..=1 {
        for dx in -1..=1 {
            let near = [column[0] + dx, column[1] + dz];
            for level in column_face_stacks(world, near) {
                world.dirty.insert(slab_of_column(near, level));
            }
        }
    }
}

pub fn drop_the_mesh(commands: &mut Commands, entity: Entity) {
    commands.queue(move |_, resources| {
        mesh_state_mark_entity_removed(resources.res_mut::<MeshRenderState>(), entity);
    });
    commands.despawn(entity);
}

pub fn unload_column(world: &mut VoxelWorld, column: ColumnKey) {
    world.surface.remove(&column);
    world.tops.remove(&column);
    for key in chunks_of(column) {
        world.chunks.remove(&key);
        world.markers.remove(&key);
    }
    for stack in column_stacks() {
        world.dirty.remove(&slab_of_column(column, stack));
    }
    world.edits += 1;
    world.sight += 1;
}

pub fn finish_column(world: &mut VoxelWorld, column: ColumnKey) {
    apply_the_edits(world, column);
    let heights = measure_surface(world, column);
    world.tops.insert(column, column_top(&heights));
    world.surface.insert(column, Arc::new(heights));
    let blocks = world.blocks.clone();
    let mut empty = Vec::new();
    let mut shared = Vec::new();
    for key in chunks_of(column) {
        let Some(chunk) = world.chunks.get_mut(&key) else {
            continue;
        };
        let first = chunk.blocks[0];
        let same = chunk.blocks.iter().all(|&block| block == first);
        if same && first == AIR {
            empty.push(key);
            continue;
        }
        chunk.solid = chunk.blocks.iter().all(|&block| block != AIR);
        chunk.stage = ChunkStage::Decorated;
        chunk.openings = chunk_openings(chunk, &blocks);
        if same {
            shared.push((key, first));
        }
    }
    for key in empty {
        world.chunks.remove(&key);
    }
    for (key, value) in shared {
        let template = match world.uniform.get(&value) {
            Some(held) => held.clone(),
            None => {
                let Some(chunk) = world.chunks.get(&key) else {
                    continue;
                };
                let held = chunk.blocks.clone();
                world.uniform.insert(value, held.clone());
                held
            }
        };
        if let Some(chunk) = world.chunks.get_mut(&key) {
            chunk.blocks = template;
        }
    }
    world.sight += 1;
    mark_column_dirty(world, column);
    world.edits += 1;
}

fn apply_the_edits(world: &mut VoxelWorld, column: ColumnKey) {
    for key in chunks_of(column) {
        let Some(edited) = world.edited.get(&key).cloned() else {
            continue;
        };
        let origin = IVec3::new(
            key[0] * data::CHUNK,
            key[1] * data::CHUNK,
            key[2] * data::CHUNK,
        );
        for (index, block) in edited {
            let inside = IVec3::new(
                index as i32 % data::CHUNK,
                index as i32 / (data::CHUNK * data::CHUNK),
                (index as i32 / data::CHUNK) % data::CHUNK,
            );
            write_block(world, origin + inside, block);
        }
    }
}

pub fn dig_the_world<D: Dig>(
    orders: &mut D::Orders,
    tools: &mut D::Tools,
    world: &mut VoxelWorld,
    columns: &[ColumnKey],
) {
    for &column in columns {
        let mut edited = HashMap::new();
        for key in chunks_of(column) {
            if let Some(changes) = world.edited.get(&key) {
                edited.insert(key, changes.clone());
            }
        }
        let order = D::order(orders, column);
        let mut private = VoxelWorld {
            blocks: world.blocks.clone(),
            edited,
            ..Default::default()
        };
        let report = D::dig(order, tools, &mut private, column);
        finish_column(&mut private, column);
        let surface = private
            .surface
            .remove(&column)
            .unwrap_or_else(|| Arc::new(vec![data::SURFACE_NONE; data::CHUNK_FACES]));
        for (key, chunk) in std::mem::take(&mut private.chunks) {
            world.chunks.insert(key, chunk);
        }
        world.tops.insert(column, column_top(&surface));
        world.surface.insert(column, surface);
        for (key, found) in std::mem::take(&mut private.markers) {
            world.markers.entry(key).or_default().extend(found);
        }
        D::settle(orders, world, column, report);
    }
}

pub fn hand_out_the_column<D: Dig>(
    world: &VoxelWorld,
    orders: &mut D::Orders,
    diggers: &mut Workers<ColumnJob<D>>,
    column: ColumnKey,
    rank: i32,
) {
    let mut edited = HashMap::new();
    for key in chunks_of(column) {
        if let Some(changes) = world.edited.get(&key) {
            edited.insert(key, changes.clone());
        }
    }
    let job = ColumnJob {
        column,
        order: D::order(orders, column),
        edited,
        blocks: world.blocks.clone(),
    };
    workers_submit(diggers, job, rank);
}

pub fn place_block(
    world: &mut VoxelWorld,
    position: IVec3,
    block: u8,
    body: Option<(Vec3, Vec3)>,
) -> bool {
    if !in_bounds(position) || !world.blocks.replaceable(block_at(world, position)) {
        return false;
    }
    let overlaps = |spot: IVec3| body.is_some_and(|(min, max)| box_overlaps_block(min, max, spot));
    if world.blocks.tall(block) {
        let above = position + IVec3::new(0, 1, 0);
        if !is_solid_at(world, position - IVec3::new(0, 1, 0))
            || !world.blocks.replaceable(block_at(world, above))
            || overlaps(position)
            || overlaps(above)
        {
            return false;
        }
        set_block(world, position, block);
        set_block(world, above, block);
        return true;
    }
    if world.blocks.solid(block) && overlaps(position) {
        return false;
    }
    if let Some(support) = world.blocks.support(block)
        && !is_solid_at(
            world,
            position + IVec3::new(support[0], support[1], support[2]),
        )
    {
        return false;
    }
    set_block(world, position, block);
    true
}

pub fn swing_the_block(world: &mut VoxelWorld, block: IVec3) {
    let up = IVec3::new(0, 1, 0);
    let kind = block_at(world, block);
    if !world.blocks.tall(kind) {
        if let Some(swung) = world.blocks.look(kind).swung {
            set_block(world, block, swung);
        }
        return;
    }
    let lowest = if block_at(world, block - up) == kind {
        block - up
    } else {
        block
    };
    for step in 0..2 {
        let position = lowest + up * step;
        let Some(swung) = world.blocks.look(block_at(world, position)).swung else {
            continue;
        };
        set_block(world, position, swung);
    }
}

pub fn reset_the_world<D: Dig>(storage: &mut Storage, resources: &mut ResourceMap) {
    let (slabs, names): (Vec<Entity>, Vec<String>) = {
        let meshes = resources.res::<Meshes>();
        (
            meshes.entities.values().copied().collect(),
            meshes
                .entities
                .keys()
                .map(|(key, layer)| slab_mesh_name(*key, *layer))
                .collect(),
        )
    };
    despawn_entities_with_cache_cleanup(storage, resources, &slabs);
    let falls: Vec<Entity> = resources
        .res::<Falling>()
        .blocks
        .iter()
        .map(|fall| fall.entity)
        .collect();
    despawn_entities_with_cache_cleanup(storage, resources, &falls);
    resources.res_mut::<Falling>().blocks.clear();
    {
        let cache = resources.res_mut::<MeshCache>();
        for name in &names {
            mesh_cache_release(cache, name);
        }
    }
    workers_clear(resources.res_mut::<Workers<ColumnJob<D>>>());
    workers_clear(resources.res_mut::<Workers<Ground>>());
    let fresh = resources.res::<VoxelWorld>().same_blocks();
    *resources.res_mut::<VoxelWorld>() = fresh;
    *resources.res_mut::<Meshes>() = Meshes::default();
    *resources.res_mut::<Streaming>() = Streaming::default();
    *resources.res_mut::<Sight>() = Sight::default();
}
