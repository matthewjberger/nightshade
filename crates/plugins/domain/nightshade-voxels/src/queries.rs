use crate::contract::{
    AIR, Blocks, Cell, ChunkKey, ChunkLayer, ColumnKey, FULL_LEVEL, Face, Hit, SlabKey, Slide,
};
use crate::data;
use crate::resources::VoxelSettings;
use crate::resources::{Chunk, VoxelWorld};
use nalgebra_glm::{IVec3, Vec2, Vec3};
use nightshade_render::data::bounding_volume::{BoundingVolume, OrientedBoundingBox};
use nightshade_render::data::geometry::{Mesh, Vertex};

pub fn in_bounds(position: IVec3) -> bool {
    position.y >= data::FLOOR_Y && position.y < data::WORLD_Y
}

pub fn column_levels() -> std::ops::Range<i32> {
    data::FLOOR_LEVEL..data::TOP_LEVEL
}

pub fn column_stacks() -> std::ops::Range<i32> {
    data::FLOOR_LEVEL.div_euclid(data::SLAB_LEVELS)
        ..(data::TOP_LEVEL - 1).div_euclid(data::SLAB_LEVELS) + 1
}

pub fn block_index(position: IVec3) -> usize {
    let x = position.x.rem_euclid(data::CHUNK);
    let y = position.y.rem_euclid(data::CHUNK);
    let z = position.z.rem_euclid(data::CHUNK);
    ((y * data::CHUNK + z) * data::CHUNK + x) as usize
}

pub fn chunk_of(position: IVec3) -> ChunkKey {
    [
        position.x.div_euclid(data::CHUNK),
        position.y.div_euclid(data::CHUNK),
        position.z.div_euclid(data::CHUNK),
    ]
}

pub fn column_of(x: i32, z: i32) -> ColumnKey {
    [x.div_euclid(data::CHUNK), z.div_euclid(data::CHUNK)]
}

pub fn column_settled(world: &VoxelWorld, column: ColumnKey) -> bool {
    world.surface.contains_key(&column)
}

pub fn slab_of(x: i32, y: i32, z: i32) -> SlabKey {
    [
        x.div_euclid(data::SLAB_SPAN),
        y.div_euclid(data::SLAB_HIGH),
        z.div_euclid(data::SLAB_SPAN),
    ]
}

pub fn slab_of_chunk(chunk: ChunkKey) -> SlabKey {
    [
        chunk[0].div_euclid(data::SLAB),
        chunk[1].div_euclid(data::SLAB_LEVELS),
        chunk[2].div_euclid(data::SLAB),
    ]
}

pub fn slab_of_column(column: ColumnKey, level: i32) -> SlabKey {
    [
        column[0].div_euclid(data::SLAB),
        level,
        column[1].div_euclid(data::SLAB),
    ]
}

pub fn slab_columns(slab: SlabKey) -> impl Iterator<Item = ColumnKey> {
    (0..data::SLAB).flat_map(move |step_x| {
        (0..data::SLAB)
            .map(move |step_z| [slab[0] * data::SLAB + step_x, slab[2] * data::SLAB + step_z])
    })
}

pub fn slab_chunks(slab: SlabKey) -> impl Iterator<Item = ChunkKey> {
    (0..data::SLAB).flat_map(move |step_x| {
        (0..data::SLAB).flat_map(move |step_z| {
            (0..data::SLAB_LEVELS).map(move |step_y| {
                [
                    slab[0] * data::SLAB + step_x,
                    slab[1] * data::SLAB_LEVELS + step_y,
                    slab[2] * data::SLAB + step_z,
                ]
            })
        })
    })
}

pub fn slab_ready(world: &VoxelWorld, slab: SlabKey) -> bool {
    let base = [slab[0] * data::SLAB - 1, slab[2] * data::SLAB - 1];
    (0..data::SLAB_COLUMNS).all(|step_x| {
        (0..data::SLAB_COLUMNS)
            .all(|step_z| column_settled(world, [base[0] + step_x, base[1] + step_z]))
    })
}

pub fn surface_index(x: i32, z: i32) -> usize {
    (x.rem_euclid(data::CHUNK) * data::CHUNK + z.rem_euclid(data::CHUNK)) as usize
}

pub fn slab_mesh_name(key: SlabKey, layer: ChunkLayer) -> String {
    let suffix = match layer {
        ChunkLayer::Solid => "solid".to_string(),
        ChunkLayer::Cutout => "cutout".to_string(),
        ChunkLayer::Transparent => "transparent".to_string(),
        ChunkLayer::Liquid(liquid) => format!("liquid{liquid}"),
    };
    format!(
        "{}_{}_{}_{}_{suffix}",
        data::MESH_PREFIX,
        key[0],
        key[1],
        key[2]
    )
}

pub fn in_blocks(settings: &VoxelSettings, position: Vec3) -> Vec3 {
    position / settings.block_size
}

pub fn in_world(settings: &VoxelSettings, position: Vec3) -> Vec3 {
    position * settings.block_size
}

pub fn layer_material<'a>(
    settings: &'a VoxelSettings,
    blocks: &'a Blocks,
    layer: ChunkLayer,
) -> &'a str {
    match layer {
        ChunkLayer::Solid => &settings.solid_material,
        ChunkLayer::Cutout => &settings.cutout_material,
        ChunkLayer::Transparent => &settings.transparent_material,
        ChunkLayer::Liquid(liquid) => blocks
            .liquids
            .get(liquid as usize)
            .map(|held| held.material.as_str())
            .unwrap_or(""),
    }
}

pub fn view_rank(across: i32, along: i32, forward: Vec3, behind: i32) -> i32 {
    let reach = across * across + along * along;
    if reach == 0 {
        return 0;
    }
    let flat = Vec2::new(forward.x, forward.z);
    let length = flat.norm();
    if length < 0.0001 {
        return reach;
    }
    let ahead = (flat.x * across as f32 + flat.y * along as f32) / (length * (reach as f32).sqrt());
    if ahead >= 0.15 { reach } else { reach * behind }
}

pub fn with_bounds(mesh: Mesh) -> Mesh {
    if mesh.vertices.is_empty() {
        return mesh;
    }
    let mut lowest = Vec3::new(f32::MAX, f32::MAX, f32::MAX);
    let mut highest = Vec3::new(f32::MIN, f32::MIN, f32::MIN);
    for vertex in &mesh.vertices {
        let position = Vec3::new(vertex.position[0], vertex.position[1], vertex.position[2]);
        lowest = lowest.inf(&position);
        highest = highest.sup(&position);
    }
    let bounds = BoundingVolume::new(
        OrientedBoundingBox::from_aabb(lowest, highest),
        (highest - lowest).norm() * 0.5,
    );
    Mesh {
        bounding_volume: Some(bounds),
        ..mesh
    }
}

fn chunk_cell_index(cell: IVec3) -> usize {
    ((cell.y * data::CHUNK + cell.z) * data::CHUNK + cell.x) as usize
}

pub fn chunk_openings(chunk: &Chunk, blocks: &Blocks) -> u64 {
    let side = data::CHUNK;
    let last = side - 1;
    let open = |cell: IVec3| -> bool { !blocks.occludes(chunk.blocks[chunk_cell_index(cell)]) };
    let mut seen = vec![false; data::CHUNK_BLOCKS];
    let mut stack: Vec<IVec3> = Vec::new();
    let mut openings = 0;
    for start_y in 0..side {
        for start_z in 0..side {
            for start_x in 0..side {
                let start = IVec3::new(start_x, start_y, start_z);
                let index = chunk_cell_index(start);
                if seen[index] || !open(start) {
                    continue;
                }
                let mut touched = 0_u8;
                seen[index] = true;
                stack.push(start);
                while let Some(cell) = stack.pop() {
                    if cell.y == last {
                        touched |= 1;
                    }
                    if cell.y == 0 {
                        touched |= 2;
                    }
                    if cell.z == 0 {
                        touched |= 4;
                    }
                    if cell.z == last {
                        touched |= 8;
                    }
                    if cell.x == last {
                        touched |= 16;
                    }
                    if cell.x == 0 {
                        touched |= 32;
                    }
                    for step in [
                        IVec3::new(1, 0, 0),
                        IVec3::new(-1, 0, 0),
                        IVec3::new(0, 1, 0),
                        IVec3::new(0, -1, 0),
                        IVec3::new(0, 0, 1),
                        IVec3::new(0, 0, -1),
                    ] {
                        let next = cell + step;
                        if !(0..side).contains(&next.x)
                            || !(0..side).contains(&next.y)
                            || !(0..side).contains(&next.z)
                        {
                            continue;
                        }
                        let held = chunk_cell_index(next);
                        if seen[held] || !open(next) {
                            continue;
                        }
                        seen[held] = true;
                        stack.push(next);
                    }
                }
                for first in 0..6 {
                    if touched & (1 << first) == 0 {
                        continue;
                    }
                    for second in 0..6 {
                        if touched & (1 << second) == 0 {
                            continue;
                        }
                        openings |= 1_u64 << (first * 6 + second);
                    }
                }
            }
        }
    }
    openings
}

pub fn block_at(world: &VoxelWorld, position: IVec3) -> u8 {
    if !in_bounds(position) {
        return AIR;
    }
    match world.chunks.get(&chunk_of(position)) {
        Some(chunk) => chunk.blocks[block_index(position)],
        None => AIR,
    }
}

pub fn chunks_of(column: ColumnKey) -> impl Iterator<Item = ChunkKey> {
    column_levels().map(move |layer| [column[0], layer, column[1]])
}

pub fn column_top(heights: &[i16]) -> i32 {
    heights
        .iter()
        .copied()
        .max()
        .map(|top| top as i32)
        .unwrap_or(data::FLOOR_Y)
}

pub fn column_face_stacks(world: &VoxelWorld, column: ColumnKey) -> std::ops::RangeInclusive<i32> {
    let top = world
        .tops
        .get(&column)
        .copied()
        .unwrap_or(data::WORLD_Y - 1);
    let stacks = column_stacks();
    let highest = (top.div_euclid(data::SLAB_HIGH) + 1).min(stacks.end - 1);
    stacks.start..=highest
}

pub fn measure_surface(world: &VoxelWorld, column: ColumnKey) -> Vec<i16> {
    let side = data::CHUNK;
    let mut heights = vec![data::SURFACE_NONE; data::CHUNK_FACES];
    let mut found = 0;
    for level in column_levels().rev() {
        let Some(chunk) = world.chunks.get(&[column[0], level, column[1]]) else {
            continue;
        };
        for local_y in (0..side).rev() {
            if found == data::CHUNK_FACES {
                return heights;
            }
            for local_z in 0..side {
                for local_x in 0..side {
                    let spot = (local_x * side + local_z) as usize;
                    if heights[spot] > data::SURFACE_NONE {
                        continue;
                    }
                    let cell = ((local_y * side + local_z) * side + local_x) as usize;
                    if world.blocks.occludes(chunk.blocks[cell]) {
                        heights[spot] = (level * side + local_y) as i16;
                        found += 1;
                    }
                }
            }
        }
    }
    heights
}

pub fn view_sector(forward: Vec3) -> i32 {
    let angle = forward.z.atan2(forward.x);
    (angle / std::f32::consts::TAU * data::VIEW_SECTORS).round() as i32
}

pub fn opening_bit(entry: usize, exit: usize) -> u64 {
    1 << (entry * 6 + exit)
}

pub fn standing_chunk(eye: Vec3) -> ChunkKey {
    let key = chunk_of(IVec3::new(
        eye.x.floor() as i32,
        eye.y.floor() as i32,
        eye.z.floor() as i32,
    ));
    [
        key[0],
        key[1].clamp(data::FLOOR_LEVEL, data::TOP_LEVEL - 1),
        key[2],
    ]
}

pub fn cave_cell(key: ChunkKey, center: ChunkKey) -> Option<usize> {
    let span = data::CAVE_SPAN;
    let across = key[0] - center[0] + data::CAVE_SIGHT;
    let level = key[1] - center[1] + data::CAVE_SIGHT;
    let along = key[2] - center[2] + data::CAVE_SIGHT;
    if !(0..span).contains(&across) || !(0..span).contains(&level) || !(0..span).contains(&along) {
        return None;
    }
    Some(((across * span + level) * span + along) as usize)
}

pub fn is_solid_at(world: &VoxelWorld, position: IVec3) -> bool {
    if position.y < data::FLOOR_Y {
        return true;
    }
    world.blocks.solid(block_at(world, position))
}

pub fn surface_height(world: &VoxelWorld, x: i32, z: i32) -> Option<i32> {
    (data::FLOOR_Y + 1..data::WORLD_Y)
        .rev()
        .find(|&y| block_at(world, IVec3::new(x, y, z)) != AIR)
}

pub fn ground_height(world: &VoxelWorld, x: i32, z: i32) -> Option<i32> {
    (data::FLOOR_Y..data::WORLD_Y)
        .rev()
        .find(|&y| is_solid_at(world, IVec3::new(x, y, z)))
}

pub fn walkable(world: &VoxelWorld, x: i32, z: i32) -> Option<i32> {
    let ground = ground_height(world, x, z)?;
    passable(world, IVec3::new(x, ground + 1, z)).then_some(ground)
}

pub fn blocks_touching(min: Vec3, max: Vec3) -> impl Iterator<Item = IVec3> {
    let low = IVec3::new(
        min.x.floor() as i32,
        min.y.floor() as i32,
        min.z.floor() as i32,
    );
    let high = IVec3::new(
        (max.x - 1.0e-4).floor() as i32,
        (max.y - 1.0e-4).floor() as i32,
        (max.z - 1.0e-4).floor() as i32,
    );
    (low.x..=high.x).flat_map(move |x| {
        (low.y..=high.y).flat_map(move |y| (low.z..=high.z).map(move |z| IVec3::new(x, y, z)))
    })
}

pub fn box_hits_solid(world: &VoxelWorld, min: Vec3, max: Vec3) -> bool {
    blocks_touching(min, max).any(|position| is_solid_at(world, position))
}

pub fn box_in_liquid(world: &VoxelWorld, min: Vec3, max: Vec3) -> Option<u8> {
    blocks_touching(min, max).find_map(|position| {
        world
            .blocks
            .fill(block_at(world, position))
            .filter(|fill| fill.level == FULL_LEVEL)
            .map(|fill| fill.liquid)
    })
}

pub fn box_overlaps_block(min: Vec3, max: Vec3, block: IVec3) -> bool {
    let block_min = Vec3::new(block.x as f32, block.y as f32, block.z as f32);
    let block_max = block_min + Vec3::new(1.0, 1.0, 1.0);
    min.x < block_max.x
        && max.x > block_min.x
        && min.y < block_max.y
        && max.y > block_min.y
        && min.z < block_max.z
        && max.z > block_min.z
}

pub fn liquid_near(world: &VoxelWorld, block: IVec3, reach: i32, liquid: u8) -> bool {
    for across in -reach..=reach {
        for along in -reach..=reach {
            for step in [0, -1] {
                let beside = block + IVec3::new(across, step, along);
                if world
                    .blocks
                    .fill(block_at(world, beside))
                    .is_some_and(|fill| fill.liquid == liquid)
                {
                    return true;
                }
            }
        }
    }
    false
}

pub fn raycast(world: &VoxelWorld, origin: Vec3, direction: Vec3, reach: f32) -> Option<Hit> {
    march(world, origin, direction, reach, |blocks, kind| {
        kind != AIR && blocks.level(kind) != FULL_LEVEL
    })
}

pub fn raycast_to_water(
    world: &VoxelWorld,
    origin: Vec3,
    direction: Vec3,
    reach: f32,
) -> Option<Hit> {
    march(world, origin, direction, reach, |_, kind| kind != AIR)
}

fn march(
    world: &VoxelWorld,
    origin: Vec3,
    direction: Vec3,
    reach: f32,
    stops: impl Fn(&Blocks, u8) -> bool,
) -> Option<Hit> {
    let mut block = IVec3::new(
        origin.x.floor() as i32,
        origin.y.floor() as i32,
        origin.z.floor() as i32,
    );
    let step = IVec3::new(
        direction.x.signum() as i32,
        direction.y.signum() as i32,
        direction.z.signum() as i32,
    );
    let delta = Vec3::new(
        (1.0 / direction.x).abs(),
        (1.0 / direction.y).abs(),
        (1.0 / direction.z).abs(),
    );
    let mut next = Vec3::new(
        boundary_distance(origin.x, direction.x, block.x),
        boundary_distance(origin.y, direction.y, block.y),
        boundary_distance(origin.z, direction.z, block.z),
    );
    let mut normal = IVec3::zeros();
    let mut traveled = 0.0;
    while traveled <= reach {
        let kind = block_at(world, block);
        if stops(&world.blocks, kind) && in_bounds(block) {
            return Some(Hit {
                block,
                normal,
                kind,
            });
        }
        if next.x < next.y && next.x < next.z {
            traveled = next.x;
            next.x += delta.x;
            block.x += step.x;
            normal = IVec3::new(-step.x, 0, 0);
        } else if next.y < next.z {
            traveled = next.y;
            next.y += delta.y;
            block.y += step.y;
            normal = IVec3::new(0, -step.y, 0);
        } else {
            traveled = next.z;
            next.z += delta.z;
            block.z += step.z;
            normal = IVec3::new(0, 0, -step.z);
        }
    }
    None
}

fn boundary_distance(origin: f32, direction: f32, block: i32) -> f32 {
    if direction > 0.0 {
        (block as f32 + 1.0 - origin) / direction
    } else if direction < 0.0 {
        (origin - block as f32) / -direction
    } else {
        f32::INFINITY
    }
}

pub fn liquid_surface_near(world: &VoxelWorld, place: Vec3) -> Option<f32> {
    let column = IVec3::new(
        place.x.floor() as i32,
        place.y.floor() as i32,
        place.z.floor() as i32,
    );
    for step in [1, 0, -1, -2] {
        let here = column + IVec3::new(0, step, 0);
        if world.blocks.is_liquid(block_at(world, here))
            && !world
                .blocks
                .is_liquid(block_at(world, here + IVec3::new(0, 1, 0)))
        {
            return Some(here.y as f32 + 1.0);
        }
    }
    None
}

pub fn slide_box(world: &VoxelWorld, position: Vec3, low: Vec3, high: Vec3, motion: Vec3) -> Slide {
    let mut place = position;
    let mut blocked = [false; 3];
    let mut on_ground = false;
    for axis in 0..3 {
        let mut moved = place;
        moved[axis] += motion[axis];
        if !box_hits_solid(world, moved + low, moved + high) {
            place = moved;
            continue;
        }
        blocked[axis] = true;
        if axis != 1 {
            moved[axis] = place[axis];
            place = moved;
            continue;
        }
        if motion.y < 0.0 {
            on_ground = true;
            let bottom = place.y + motion.y + low.y;
            moved.y = bottom.floor() + 1.0 + 1.0e-3 - low.y;
            if box_hits_solid(world, moved + low, moved + high) {
                moved.y = place.y;
            }
        } else {
            moved.y = place.y;
        }
        place = moved;
    }
    Slide {
        position: place,
        on_ground,
        blocked,
    }
}

pub fn facing_of(yaw: f32) -> IVec3 {
    let forward = Vec3::new(-yaw.sin(), 0.0, -yaw.cos());
    if forward.x.abs() > forward.z.abs() {
        IVec3::new(-forward.x.signum() as i32, 0, 0)
    } else {
        IVec3::new(0, 0, -forward.z.signum() as i32)
    }
}

pub fn passable(world: &VoxelWorld, position: IVec3) -> bool {
    let block = block_at(world, position);
    !world.blocks.solid(block) && !world.blocks.is_liquid(block)
}

pub fn footing(world: &VoxelWorld, x: i32, z: i32, from: f32, rise: i32, fall: i32) -> Option<i32> {
    let start = from.floor() as i32 + rise;
    let stop = start - rise - fall;
    (stop..=start).rev().find(|&y| {
        is_solid_at(world, IVec3::new(x, y, z))
            && passable(world, IVec3::new(x, y + 1, z))
            && passable(world, IVec3::new(x, y + 2, z))
    })
}

pub fn cube_mesh(blocks: &Blocks, cells: impl Fn(Face) -> Cell, low: f32, high: f32) -> Mesh {
    let mut vertices = Vec::with_capacity(24);
    let mut indices = Vec::with_capacity(36);
    let uvs = [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]];
    for face in Face::ALL {
        let normal = face.normal();
        let normal = Vec3::new(normal[0] as f32, normal[1] as f32, normal[2] as f32);
        let cell = blocks.cell_index(cells(face));
        let base = vertices.len() as u32;
        for (index, corner) in cube_face(face, low, high).iter().enumerate() {
            vertices.push(Vertex::with_all(
                *corner,
                normal,
                uvs[index],
                [cell, 1.0],
                [1.0, 0.0, 0.0, 1.0],
                [1.0, 1.0, 1.0, 1.0],
            ));
        }
        indices.extend([0, 1, 2, 0, 2, 3].iter().map(|offset| base + offset));
    }
    with_bounds(Mesh::new(vertices, indices))
}

fn cube_face(face: Face, low: f32, high: f32) -> [Vec3; 4] {
    let (l, h) = (low, high);
    match face {
        Face::South => [
            Vec3::new(l, l, h),
            Vec3::new(h, l, h),
            Vec3::new(h, h, h),
            Vec3::new(l, h, h),
        ],
        Face::North => [
            Vec3::new(h, l, l),
            Vec3::new(l, l, l),
            Vec3::new(l, h, l),
            Vec3::new(h, h, l),
        ],
        Face::East => [
            Vec3::new(h, l, h),
            Vec3::new(h, l, l),
            Vec3::new(h, h, l),
            Vec3::new(h, h, h),
        ],
        Face::West => [
            Vec3::new(l, l, l),
            Vec3::new(l, l, h),
            Vec3::new(l, h, h),
            Vec3::new(l, h, l),
        ],
        Face::Top => [
            Vec3::new(l, h, h),
            Vec3::new(h, h, h),
            Vec3::new(h, h, l),
            Vec3::new(l, h, l),
        ],
        Face::Bottom => [
            Vec3::new(l, l, l),
            Vec3::new(h, l, l),
            Vec3::new(h, l, h),
            Vec3::new(l, l, h),
        ],
    }
}
