use crate::data::{Biome, BlockKind, Building, Land, VILLAGE, block_table, building_block};
use crate::queries::{
    biome_and_rock, biome_at, biome_weights, block_at, block_index, cave_field, chip_at, chunk_of,
    chunks_of, column_levels, column_of, column_settled, dungeon_at, dungeon_chest, footprint_base,
    footprint_flatten, ground_top, hash, in_bounds, near_building, region_of, reserved,
    rock_density, shelf_height, slab_of, surface_height, villages_near, walkable,
};
use crate::resources::{
    ChunkStage, ColumnKey, ColumnOrder, ColumnReport, Digging, Footprint, Generation, Heights,
    Noises, Player, Save, Saved, Village, VoxelWorld, WorldPlan,
};
use crate::theme;
use nalgebra_glm::IVec3;
use nightshade::prelude::*;
use nightshade_random::prelude::*;
use nightshade_voxels::prelude::{Dig, dig_the_world, finish_column, write_block};
use noise::NoiseFn;
use std::collections::HashMap;
use std::sync::Arc;

const PASSES: [fn(&mut VoxelWorld, &mut Generation, ColumnKey); 9] = [
    terrain, water, caves, dungeon, ores, carried, village, trees, flowers,
];

pub fn place_player(
    mut player: ResMut<Player>,
    mut world: ResMut<VoxelWorld>,
    mut generation: ResMut<Generation>,
    save: Res<Save>,
) {
    world.blocks = Arc::new(block_table());
    stand_the_player_up(
        &mut player,
        &mut world,
        &mut generation,
        save.found.as_ref(),
    );
}

pub fn stand_the_player_up(
    player: &mut Player,
    world: &mut VoxelWorld,
    generation: &mut Generation,
    found: Option<&Saved>,
) {
    generation.noises = Noises::new(generation.seed);
    let saved = found.map(|found| {
        (
            found.position[0].floor() as i32,
            found.position[2].floor() as i32,
        )
    });
    let spawn = match saved {
        Some(square) => square,
        None => choose_spawn(generation),
    };
    generation.spawn = Some(spawn);
    let center = column_of(spawn.0, spawn.1);
    for dz in -theme::SPAWN_RADIUS..=theme::SPAWN_RADIUS {
        for dx in -theme::SPAWN_RADIUS..=theme::SPAWN_RADIUS {
            raise_the_column(world, generation, [center[0] + dx, center[1] + dz]);
        }
    }
    if let Some(found) = found {
        player.position =
            nalgebra_glm::vec3(found.position[0], found.position[1], found.position[2]);
        player.home = nalgebra_glm::vec3(found.home[0], found.home[1], found.home[2]);
        player.yaw = found.yaw;
        player.pitch = found.pitch;
        player.fall_from = found.position[1];
        let standing = walkable(
            world,
            found.position[0].floor() as i32,
            found.position[2].floor() as i32,
        );
        if let Some(ground) = standing
            && found.position[1] < ground as f32
        {
            player.position.y = ground as f32 + 1.0;
            player.home.y = player.position.y;
            player.fall_from = player.position.y;
        }
        return;
    }
    let (x, z, ground) = stand_near(world, generation, spawn);
    generation.spawn = Some((x, z));
    player.position = nalgebra_glm::vec3(x as f32 + 0.5, ground as f32 + 1.0, z as f32 + 0.5);
    player.home = player.position;
    player.velocity = nalgebra_glm::Vec3::zeros();
    let facing = villages_near(generation, x, z)
        .next()
        .map(|village| village.square)
        .unwrap_or((x, z + 1));
    let (dx, dz) = ((facing.0 - x) as f32, (facing.1 - z) as f32);
    player.yaw = if dx.abs() + dz.abs() > 0.5 {
        (-dx).atan2(-dz)
    } else {
        0.0
    };
    player.pitch = -0.15;
}

fn stand_near(world: &VoxelWorld, generation: &Generation, spawn: (i32, i32)) -> (i32, i32, i32) {
    let mut candidates: Vec<(i32, i32)> = Vec::new();
    if let Some(village) = villages_near(generation, spawn.0, spawn.1).next() {
        candidates.push(village.square);
        let mut roads = village.roads.clone();
        roads.sort_by_key(|(x, z)| (x - village.square.0).abs() + (z - village.square.1).abs());
        candidates.extend(roads);
    }
    candidates.extend(spiral(14).map(|(dx, dz)| (spawn.0 + dx, spawn.1 + dz)));
    let found = candidates.into_iter().find_map(|(x, z)| {
        let ground = walkable(world, x, z)?;
        let head = IVec3::new(x, ground + 2, z);
        (ground > theme::SEA_LEVEL
            && block_at(world, head) == BlockKind::Air
            && !near_building(generation, x, z, 1))
        .then_some((x, z, ground))
    });
    found.unwrap_or((
        spawn.0,
        spawn.1,
        surface_height(world, spawn.0, spawn.1).unwrap_or(theme::SEA_LEVEL),
    ))
}

fn choose_spawn(generation: &mut Generation) -> (i32, i32) {
    load_villages_near(generation, [0, 0]);
    let home = villages_near(generation, 0, 0)
        .min_by_key(|village| village.center.0.abs() + village.center.1.abs())
        .map(|village| village.square);
    let (center, reach) = match home {
        Some(square) => ((square.0, square.1 + 2), theme::SPAWN_SEARCH),
        None => ((0, 0), theme::SPAWN_SEARCH_WIDE),
    };
    let found = spiral(reach).find_map(|(dx, dz)| {
        let (x, z) = (center.0 + dx, center.1 + dz);
        let height = column_height(generation, x, z);
        (!reserved(generation, x, z) && height > theme::SEA_LEVEL + 1).then_some((x, z))
    });
    found.unwrap_or(center)
}

impl Dig for Digging {
    type Orders = Generation;
    type Order = ColumnOrder;
    type Tools = Generation;
    type Report = ColumnReport;

    fn order(generation: &mut Generation, column: ColumnKey) -> ColumnOrder {
        let mut spill = HashMap::new();
        for key in chunks_of(column) {
            if let Some(waiting) = generation.spill.remove(&key) {
                spill.insert(key, waiting);
            }
        }
        ColumnOrder {
            seed: generation.seed,
            plan: generation.plan,
            villages: villages_around(generation, column),
            spill,
        }
    }

    fn dig(
        order: ColumnOrder,
        tools: &mut Generation,
        world: &mut VoxelWorld,
        column: ColumnKey,
    ) -> ColumnReport {
        if tools.seed != order.seed {
            tools.seed = order.seed;
            tools.noises = Noises::new(order.seed);
            tools.villages.clear();
        }
        tools.plan = order.plan;
        tools.villages.extend(order.villages);
        tools.spill = order.spill;
        tools.heights = None;
        dig_the_column(world, tools, column);
        ColumnReport {
            spill: std::mem::take(&mut tools.spill),
            villages: villages_around(tools, column),
            signs: signs_in(tools, column),
        }
    }

    fn settle(
        generation: &mut Generation,
        world: &mut VoxelWorld,
        column: ColumnKey,
        report: ColumnReport,
    ) -> Vec<ColumnKey> {
        for (region, found) in report.villages {
            generation.villages.entry(region).or_insert(found);
        }
        let mut touched = Vec::new();
        let mut waiting_here = HashMap::new();
        for key in chunks_of(column) {
            if let Some(waiting) = generation.spill.remove(&key) {
                waiting_here.insert(key, waiting);
            }
        }
        for (key, waiting) in waiting_here.into_iter().chain(report.spill) {
            let grown = column_settled(world, [key[0], key[2]])
                || world
                    .chunks
                    .get(&key)
                    .is_some_and(|chunk| chunk.stage >= ChunkStage::Terrain);
            if !grown {
                generation.spill.entry(key).or_default().extend(waiting);
                continue;
            }
            for (position, block, only_air) in waiting {
                if only_air && block_at(world, position) != BlockKind::Air {
                    continue;
                }
                write_block(world, position, block);
                touched.push(column_of(position.x, position.z));
            }
        }
        for (block, text) in report.signs {
            generation.signs.insert(block, text);
        }
        touched
    }
}

fn villages_around(
    generation: &Generation,
    column: ColumnKey,
) -> HashMap<(i32, i32), Option<Village>> {
    let region = region_of(column[0] * theme::CHUNK, column[1] * theme::CHUNK);
    let mut villages = HashMap::new();
    for dx in -1..=1 {
        for dz in -1..=1 {
            let wanted = (region.0 + dx, region.1 + dz);
            if let Some(found) = generation.villages.get(&wanted) {
                villages.insert(wanted, found.clone());
            }
        }
    }
    villages
}

fn signs_in(generation: &Generation, column: ColumnKey) -> Vec<([i32; 3], String)> {
    let origin = IVec3::new(column[0] * theme::CHUNK, 0, column[1] * theme::CHUNK);
    villages_near(generation, origin.x, origin.z)
        .flat_map(|village| village.signs.iter())
        .filter(|(block, _)| column_of(block[0], block[2]) == column)
        .cloned()
        .collect()
}

pub fn digest_the_world(seed: u32, radius: i32) -> (VoxelWorld, Vec<ColumnKey>) {
    let mut world = VoxelWorld {
        blocks: Arc::new(block_table()),
        ..Default::default()
    };
    let mut orders = Generation {
        seed,
        noises: Noises::new(seed),
        ..Default::default()
    };
    let mut tools = Generation {
        seed,
        noises: Noises::new(seed),
        ..Default::default()
    };
    let mut columns = Vec::new();
    for step_z in -radius..=radius {
        for step_x in -radius..=radius {
            columns.push([step_x, step_z]);
        }
    }
    dig_the_world::<Digging>(&mut orders, &mut tools, &mut world, &columns);
    (world, columns)
}

fn dig_the_column(world: &mut VoxelWorld, generation: &mut Generation, column: ColumnKey) {
    load_villages_near(generation, column);
    fill_the_heights(generation, column);
    for pass in PASSES {
        pass(world, generation, column);
    }
}

fn raise_the_column(world: &mut VoxelWorld, generation: &mut Generation, column: ColumnKey) {
    if world
        .chunks
        .get(&[column[0], 0, column[1]])
        .is_some_and(|chunk| chunk.stage >= ChunkStage::Decorated)
    {
        return;
    }
    dig_the_column(world, generation, column);
    finish_column(world, column);
    for (block, text) in signs_in(generation, column) {
        generation.signs.insert(block, text);
    }
}

fn place(
    world: &mut VoxelWorld,
    generation: &mut Generation,
    position: IVec3,
    kind: BlockKind,
    only_air: bool,
) {
    if !in_bounds(position) {
        return;
    }
    let key = chunk_of(position);
    let grown = world
        .chunks
        .get(&key)
        .is_some_and(|chunk| chunk.stage >= ChunkStage::Terrain);
    if !grown {
        generation
            .spill
            .entry(key)
            .or_default()
            .push((position, kind.index(), only_air));
        return;
    }
    if only_air && block_at(world, position) != BlockKind::Air {
        return;
    }
    write_block(world, position, kind.index());
    world
        .dirty
        .insert(slab_of(position.x, position.y, position.z));
}

fn carried(world: &mut VoxelWorld, generation: &mut Generation, column: ColumnKey) {
    for key in chunks_of(column) {
        let Some(waiting) = generation.spill.remove(&key) else {
            continue;
        };
        for (position, block, only_air) in waiting {
            if only_air && block_at(world, position) != BlockKind::Air {
                continue;
            }
            write_block(world, position, block);
        }
    }
}

fn load_villages_near(generation: &mut Generation, column: ColumnKey) {
    let region = region_of(column[0] * theme::CHUNK, column[1] * theme::CHUNK);
    for dx in -1..=1 {
        for dz in -1..=1 {
            let wanted = (region.0 + dx, region.1 + dz);
            if generation.villages.contains_key(&wanted) {
                continue;
            }
            let found = village_in_region(generation, wanted);
            generation.villages.insert(wanted, found);
        }
    }
}

fn land_height(noises: &Noises, plan: &WorldPlan, x: i32, z: i32) -> i32 {
    if plan.land == Land::Flat {
        return theme::FLAT_HEIGHT;
    }
    let point = [x as f64, z as f64];
    let roll = noises.rolling.get(point) as f32;
    let fine = noises.detail.get([x as f64 / 9.0, z as f64 / 9.0]) as f32;
    let weights = biome_weights(noises, x, z);
    let relief = match plan.land {
        Land::Islands => theme::ISLAND_RELIEF,
        Land::Peaks => theme::PEAKS_RELIEF,
        Land::Extreme => theme::EXTREME_RELIEF,
        _ => 1.0,
    };
    let mut height = 0.0;
    for (biome, weight) in Biome::ALL.iter().zip(weights) {
        let info = biome.info();
        height += weight * (info.base_height + info.relief * relief * roll);
    }
    height += fine * 1.2;
    height *= theme::HEIGHT_SCALE;
    if plan.land == Land::Islands {
        height -= theme::ISLAND_DROP * theme::HEIGHT_SCALE;
    }
    (height.round() as i32).clamp(theme::FLOOR_Y + 6, theme::WORLD_Y - plan.land.headroom())
}

fn column_height(generation: &Generation, x: i32, z: i32) -> i32 {
    if let Some(heights) = &generation.heights {
        let span = theme::CHUNK + 2;
        let offset_x = x - heights.column[0] * theme::CHUNK + 1;
        let offset_z = z - heights.column[1] * theme::CHUNK + 1;
        if (0..span).contains(&offset_x) && (0..span).contains(&offset_z) {
            return heights.values[(offset_z * span + offset_x) as usize];
        }
    }
    measured_height(generation, x, z)
}

fn measured_height(generation: &Generation, x: i32, z: i32) -> i32 {
    footprint_base(generation, x, z)
        .unwrap_or_else(|| land_height(&generation.noises, &generation.plan, x, z))
}

fn fill_the_heights(generation: &mut Generation, column: ColumnKey) {
    generation.heights = None;
    let origin = (column[0] * theme::CHUNK, column[1] * theme::CHUNK);
    let step = theme::LATTICE_STEP;
    let nodes = theme::LATTICE_NODES;
    let mut lattice = Vec::with_capacity((nodes * nodes) as usize);
    for node_z in 0..nodes {
        for node_x in 0..nodes {
            let x = origin.0 + node_x * step - step;
            let z = origin.1 + node_z * step - step;
            lattice.push(land_height(&generation.noises, &generation.plan, x, z) as f32);
        }
    }
    let span = theme::CHUNK + 2;
    let mut values = Vec::with_capacity((span * span) as usize);
    for offset_z in -1..=theme::CHUNK {
        for offset_x in -1..=theme::CHUNK {
            let (x, z) = (origin.0 + offset_x, origin.1 + offset_z);
            if let Some(base) = footprint_base(generation, x, z) {
                values.push(base);
                continue;
            }
            values.push(lattice_height(&lattice, nodes, step, offset_x, offset_z).round() as i32);
        }
    }
    let tops = vec![theme::FLOOR_Y - 1; (theme::CHUNK * theme::CHUNK) as usize];
    generation.heights = Some(Heights {
        column,
        values,
        tops,
        lattice,
    });
}

fn lattice_height(lattice: &[f32], nodes: i32, step: i32, offset_x: i32, offset_z: i32) -> f32 {
    let place_x = (offset_x + step) as f32 / step as f32;
    let place_z = (offset_z + step) as f32 / step as f32;
    let node_x = place_x.floor() as i32;
    let node_z = place_z.floor() as i32;
    let share_x = place_x - node_x as f32;
    let share_z = place_z - node_z as f32;
    let at = |across: i32, along: i32| lattice[(along * nodes + across) as usize];
    let low = at(node_x, node_z) + (at(node_x + 1, node_z) - at(node_x, node_z)) * share_x;
    let high =
        at(node_x, node_z + 1) + (at(node_x + 1, node_z + 1) - at(node_x, node_z + 1)) * share_x;
    low + (high - low) * share_z
}

fn terrain(world: &mut VoxelWorld, generation: &mut Generation, column: ColumnKey) {
    for key in chunks_of(column) {
        let chunk = world.chunks.entry(key).or_default();
        if chunk.stage < ChunkStage::Terrain {
            chunk.stage = ChunkStage::Terrain;
        }
    }
    let origin = IVec3::new(column[0] * theme::CHUNK, 0, column[1] * theme::CHUNK);
    let step = theme::LATTICE_STEP;
    let nodes = theme::LATTICE_NODES;
    let levels = theme::LATTICE_LEVELS;
    let lift = generation.plan.land.lift();
    let ground = generation.plan.land.rock();
    let Some(lattice) = generation
        .heights
        .as_ref()
        .map(|heights| heights.lattice.clone())
    else {
        return;
    };

    let mut node_rock = vec![0.0f32; (nodes * nodes) as usize];
    let mut node_biome = Vec::with_capacity((nodes * nodes) as usize);
    let mut field = vec![0.0f32; (nodes * nodes * levels) as usize];
    for node_z in 0..nodes {
        for node_x in 0..nodes {
            let x = origin.x + node_x * step - step;
            let z = origin.z + node_z * step - step;
            let spot = (node_z * nodes + node_x) as usize;
            let height = lattice[spot].round() as i32;
            let (kind, rock) = biome_and_rock(&generation.noises, x, z);
            let rock = rock * ground;
            let surface = shelf_height(&generation.noises, x, z, height, rock);
            node_rock[spot] = rock;
            node_biome.push(kind);
            for level in 0..levels {
                let y = theme::FLOOR_Y + level * step;
                field[((level * nodes + node_z) * nodes + node_x) as usize] =
                    rock_density(&generation.noises, x, y, z, surface, rock, lift);
            }
        }
    }

    let span = theme::CHUNK + 2;
    let cell_of = |offset_x: i32, offset_z: i32, y: i32| {
        (((y - theme::FLOOR_Y) * span + offset_z + 1) * span + offset_x + 1) as usize
    };
    let mix = |low: f32, high: f32, share: f32| low + (high - low) * share;
    let mut solid = vec![false; (span * span * theme::WORLD_TALL) as usize];
    for offset_z in -1..=theme::CHUNK {
        for offset_x in -1..=theme::CHUNK {
            let (x, z) = (origin.x + offset_x, origin.z + offset_z);
            let plane = column_height(generation, x, z) as f32;
            let flatten = footprint_flatten(generation, x, z);
            let place_x = (offset_x + step) as f32 / step as f32;
            let place_z = (offset_z + step) as f32 / step as f32;
            let node_x = place_x.floor() as i32;
            let node_z = place_z.floor() as i32;
            let share_x = place_x - node_x as f32;
            let share_z = place_z - node_z as f32;
            let rock_here = mix(
                mix(
                    node_rock[(node_z * nodes + node_x) as usize],
                    node_rock[(node_z * nodes + node_x + 1) as usize],
                    share_x,
                ),
                mix(
                    node_rock[((node_z + 1) * nodes + node_x) as usize],
                    node_rock[((node_z + 1) * nodes + node_x + 1) as usize],
                    share_x,
                ),
                share_z,
            )
            .clamp(0.0, theme::CHIP_ROCK);
            let corner = [
                (node_z * nodes + node_x) as usize,
                (node_z * nodes + node_x + 1) as usize,
                ((node_z + 1) * nodes + node_x) as usize,
                ((node_z + 1) * nodes + node_x + 1) as usize,
            ];
            let plane_at = |level: i32| {
                let base = (level * nodes * nodes) as usize;
                mix(
                    mix(field[base + corner[0]], field[base + corner[1]], share_x),
                    mix(field[base + corner[2]], field[base + corner[3]], share_x),
                    share_z,
                )
            };
            let mut low = plane_at(0);
            for level in 0..levels - 1 {
                let high = plane_at(level + 1);
                for sub in 0..step {
                    let y = theme::FLOOR_Y + level * step + sub;
                    let rocky = mix(low, high, sub as f32 / step as f32);
                    let mut value = rocky * (1.0 - flatten) + (plane - y as f32) * flatten;
                    if value.abs() < theme::CHIP_SHELL {
                        let chip = chip_at(&generation.noises, x, y, z);
                        value += chip * theme::CHIP_REACH * rock_here * (1.0 - flatten);
                    }
                    solid[cell_of(offset_x, offset_z, y)] = y == theme::FLOOR_Y || value > 0.0;
                }
                low = high;
            }
        }
    }

    for offset_z in -1..=theme::CHUNK {
        for offset_x in -1..=theme::CHUNK {
            clear_the_clumps(&mut solid, &cell_of, offset_x, offset_z);
        }
    }

    let solid_at = |offset_x: i32, offset_z: i32, y: i32| {
        if y < theme::FLOOR_Y {
            return true;
        }
        if y >= theme::WORLD_Y {
            return false;
        }
        solid[cell_of(offset_x, offset_z, y)]
    };
    let mut tops = vec![theme::FLOOR_Y - 1; (theme::CHUNK * theme::CHUNK) as usize];
    for local_z in 0..theme::CHUNK {
        for local_x in 0..theme::CHUNK {
            let (x, z) = (origin.x + local_x, origin.z + local_z);
            let near_x = ((local_x + step) as f32 / step as f32).round() as i32;
            let near_z = ((local_z + step) as f32 / step as f32).round() as i32;
            let biome = node_biome[(near_z * nodes + near_x) as usize].info();
            let mut depth = 0;
            let mut face = false;
            for y in (theme::FLOOR_Y..theme::WORLD_Y).rev() {
                if !solid_at(local_x, local_z, y) {
                    depth = 0;
                    continue;
                }
                if depth == 0 {
                    face = steep_ground(&solid_at, local_x, local_z, y);
                    let spot = (local_z * theme::CHUNK + local_x) as usize;
                    if tops[spot] < theme::FLOOR_Y {
                        tops[spot] = y;
                    }
                }
                let kind = if y == theme::FLOOR_Y {
                    BlockKind::Bedrock
                } else if depth == 0 {
                    if face { BlockKind::Stone } else { biome.top }
                } else if depth <= 3 {
                    if face {
                        BlockKind::Gravel
                    } else {
                        biome.filler
                    }
                } else {
                    BlockKind::Stone
                };
                let level = y.div_euclid(theme::CHUNK);
                let chunk = world
                    .chunks
                    .entry([column[0], level, column[1]])
                    .or_default();
                Arc::make_mut(&mut chunk.blocks)[block_index(IVec3::new(x, y, z))] = kind.index();
                depth += 1;
            }
        }
    }
    if let Some(heights) = generation.heights.as_mut() {
        heights.tops = tops;
    }
}

fn steep_ground(
    solid_at: &impl Fn(i32, i32, i32) -> bool,
    local_x: i32,
    local_z: i32,
    y: i32,
) -> bool {
    let mut walls = 0;
    let mut drops = 0;
    for (step_x, step_z) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
        if solid_at(local_x + step_x, local_z + step_z, y + 1) {
            walls += 1;
        }
        if !solid_at(local_x + step_x, local_z + step_z, y) {
            drops += 1;
        }
    }
    walls >= 2 || drops >= 2
}

fn clear_the_clumps(
    solid: &mut [bool],
    cell_of: &impl Fn(i32, i32, i32) -> usize,
    offset_x: i32,
    offset_z: i32,
) {
    let mut y = theme::FLOOR_Y + 1;
    while y < theme::WORLD_Y {
        if !solid[cell_of(offset_x, offset_z, y)] {
            y += 1;
            continue;
        }
        let start = y;
        while y < theme::WORLD_Y && solid[cell_of(offset_x, offset_z, y)] {
            y += 1;
        }
        if y - start > theme::CLUMP_RUN {
            continue;
        }
        let mut below = 0;
        while below < theme::CLUMP_GAP
            && start - 1 - below >= theme::FLOOR_Y
            && !solid[cell_of(offset_x, offset_z, start - 1 - below)]
        {
            below += 1;
        }
        let mut above = 0;
        while above < theme::CLUMP_GAP
            && y + above < theme::WORLD_Y
            && !solid[cell_of(offset_x, offset_z, y + above)]
        {
            above += 1;
        }
        let open_above = above >= theme::CLUMP_GAP || y + above >= theme::WORLD_Y;
        if below >= theme::CLUMP_GAP && open_above {
            for level in start..y {
                solid[cell_of(offset_x, offset_z, level)] = false;
            }
        }
    }
}

fn water(world: &mut VoxelWorld, generation: &mut Generation, column: ColumnKey) {
    let origin = IVec3::new(column[0] * theme::CHUNK, 0, column[1] * theme::CHUNK);
    let mut levels = Vec::with_capacity(theme::COLUMN_CHUNKS as usize);
    for level in column_levels() {
        levels.push(world.chunks.get(&[column[0], level, column[1]]).cloned());
    }
    let kind_at = |position: IVec3| -> BlockKind {
        let slot = position.y.div_euclid(theme::CHUNK) - theme::FLOOR_LEVEL;
        let Some(chunk) = usize::try_from(slot)
            .ok()
            .and_then(|slot| levels.get(slot))
            .and_then(|held| held.as_ref())
        else {
            return BlockKind::Air;
        };
        BlockKind::from_index(chunk.blocks[block_index(position)])
    };
    let mut wet = Vec::new();
    let mut shore = Vec::new();
    for local_z in 0..theme::CHUNK {
        for local_x in 0..theme::CHUNK {
            let (x, z) = (origin.x + local_x, origin.z + local_z);
            let frozen = biome_at(generation, x, z).is_cold();
            for y in theme::FLOOR_Y + 1..=theme::SEA_LEVEL {
                if kind_at(IVec3::new(x, y, z)) != BlockKind::Air {
                    continue;
                }
                let kind = if frozen && y == theme::SEA_LEVEL {
                    BlockKind::Ice
                } else {
                    BlockKind::Water
                };
                wet.push((IVec3::new(x, y, z), kind));
            }
            let top = (theme::FLOOR_Y + 1..theme::WORLD_Y)
                .rev()
                .find(|&y| kind_at(IVec3::new(x, y, z)) != BlockKind::Air);
            let Some(top) = top else {
                continue;
            };
            if top > theme::SEA_LEVEL + 2 {
                continue;
            }
            for y in (top - 3).max(1)..=top {
                if kind_at(IVec3::new(x, y, z)) == BlockKind::Stone {
                    shore.push(IVec3::new(x, y, z));
                }
            }
        }
    }
    drop(levels);
    for (position, kind) in wet {
        write_block(world, position, kind.index());
    }
    for position in shore {
        write_block(world, position, BlockKind::Sand.index());
    }
}

fn caves(world: &mut VoxelWorld, generation: &mut Generation, column: ColumnKey) {
    if !generation.plan.caves {
        return;
    }
    let origin = IVec3::new(column[0] * theme::CHUNK, 0, column[1] * theme::CHUNK);
    let mut ceiling = theme::FLOOR_Y;
    for local_z in 0..theme::CHUNK {
        for local_x in 0..theme::CHUNK {
            let top = ground_top(generation, origin.x + local_x, origin.z + local_z)
                .unwrap_or(theme::FLOOR_Y);
            ceiling = ceiling.max(top);
        }
    }
    if ceiling <= theme::FLOOR_Y + 8 {
        return;
    }
    let step = theme::LATTICE_STEP;
    let nodes = theme::CHUNK / step + 1;
    let levels = (ceiling - theme::FLOOR_Y) / step + 2;
    let mut field = vec![[0.0f32; 3]; (nodes * nodes * levels) as usize];
    for level in 0..levels {
        let y = theme::FLOOR_Y + level * step;
        for node_z in 0..nodes {
            for node_x in 0..nodes {
                let x = origin.x + node_x * step;
                let z = origin.z + node_z * step;
                field[((level * nodes + node_z) * nodes + node_x) as usize] =
                    cave_field(&generation.noises, x, y, z);
            }
        }
    }
    let mix = |low: f32, high: f32, share: f32| low + (high - low) * share;
    let mut carved = Vec::new();
    for local_z in 0..theme::CHUNK {
        for local_x in 0..theme::CHUNK {
            let (x, z) = (origin.x + local_x, origin.z + local_z);
            let height = ground_top(generation, x, z).unwrap_or(theme::FLOOR_Y);
            let built = reserved(generation, x, z);
            let slot = generation.noises.ravine.get([x as f64, z as f64]);
            let ravine = !built && slot.abs() < theme::RAVINE_WIDTH;
            let place_x = local_x as f32 / step as f32;
            let place_z = local_z as f32 / step as f32;
            let node_x = place_x.floor() as i32;
            let node_z = place_z.floor() as i32;
            let share_x = place_x - node_x as f32;
            let share_z = place_z - node_z as f32;
            let corner = [
                (node_z * nodes + node_x) as usize,
                (node_z * nodes + node_x + 1) as usize,
                ((node_z + 1) * nodes + node_x) as usize,
                ((node_z + 1) * nodes + node_x + 1) as usize,
            ];
            let plane_at = |level: i32| {
                let base = (level * nodes * nodes) as usize;
                let mut found = [0.0f32; 3];
                for (slot, value) in found.iter_mut().enumerate() {
                    *value = mix(
                        mix(
                            field[base + corner[0]][slot],
                            field[base + corner[1]][slot],
                            share_x,
                        ),
                        mix(
                            field[base + corner[2]][slot],
                            field[base + corner[3]][slot],
                            share_x,
                        ),
                        share_z,
                    );
                }
                found
            };
            let mut held = -1;
            let mut low = [0.0f32; 3];
            let mut high = [0.0f32; 3];
            for y in theme::FLOOR_Y + 4..(height - 4) {
                let place_y = (y - theme::FLOOR_Y) as f32 / step as f32;
                let level = place_y.floor() as i32;
                let share_y = place_y - level as f32;
                if level != held {
                    held = level;
                    low = plane_at(level);
                    high = plane_at(level + 1);
                }
                let mut carve = mix(low[0], high[0], share_y).abs() < 0.09
                    && mix(low[1], high[1], share_y).abs() < 0.16;
                if !carve && !built && y < theme::CAVERN_TOP {
                    carve = mix(low[2], high[2], share_y) > theme::CAVERN_EDGE as f32;
                }
                if !carve && ravine && y > theme::RAVINE_FLOOR && y < height - 6 {
                    carve = true;
                }
                if !carve {
                    continue;
                }
                let kind = if y <= theme::FLOOD_LEVEL {
                    BlockKind::Water
                } else {
                    BlockKind::Air
                };
                carved.push((IVec3::new(x, y, z), kind));
            }
        }
    }
    for (position, kind) in carved {
        write_block(world, position, kind.index());
    }
}
fn dungeon(world: &mut VoxelWorld, generation: &mut Generation, column: ColumnKey) {
    if !generation.plan.dungeons {
        return;
    }
    let origin = IVec3::new(column[0] * theme::CHUNK, 0, column[1] * theme::CHUNK);
    let home = region_of(origin.x, origin.z);
    for dz in -1..=1 {
        for dx in -1..=1 {
            let region = (home.0 + dx, home.1 + dz);
            let Some((corner, span)) = dungeon_at(generation.seed, region) else {
                continue;
            };
            let chest = dungeon_chest(generation.seed, region);
            let reach = -theme::DUNGEON_TUNNEL..(span + theme::DUNGEON_TUNNEL);
            for local_z in 0..theme::CHUNK {
                for local_x in 0..theme::CHUNK {
                    let (x, z) = (origin.x + local_x, origin.z + local_z);
                    let inside = (x - corner.x, z - corner.z);
                    if !reach.contains(&inside.0) || !reach.contains(&inside.1) {
                        continue;
                    }
                    if in_the_rooms(inside, span) {
                        build_room(world, IVec3::new(x, corner.y, z), inside, span, chest);
                    } else if in_the_tunnel(inside, span) {
                        for step in 1..theme::DUNGEON_TALL {
                            write_block(
                                world,
                                IVec3::new(x, corner.y + step, z),
                                BlockKind::Air.index(),
                            );
                        }
                    }
                }
            }
        }
    }
}

fn build_room(
    world: &mut VoxelWorld,
    floor: IVec3,
    inside: (i32, i32),
    span: i32,
    chest: Option<IVec3>,
) {
    let wall = room_wall(inside, span);
    for step in 0..=theme::DUNGEON_TALL {
        let kind = if step == 0 || step == theme::DUNGEON_TALL || wall {
            BlockKind::Cobblestone
        } else {
            BlockKind::Air
        };
        write_block(
            world,
            IVec3::new(floor.x, floor.y + step, floor.z),
            kind.index(),
        );
    }
    if wall {
        return;
    }
    let lamp = inside.0 % theme::DUNGEON_ROOM == 4 && inside.1 % theme::DUNGEON_ROOM == 4;
    if lamp {
        let under_the_ceiling = floor.y + theme::DUNGEON_TALL - 1;
        write_block(
            world,
            IVec3::new(floor.x, under_the_ceiling, floor.z),
            BlockKind::Glowstone.index(),
        );
    }
    let stand = IVec3::new(floor.x, floor.y + 1, floor.z);
    if chest == Some(stand) {
        write_block(world, stand, BlockKind::Chest.index());
    }
}

fn in_the_rooms(inside: (i32, i32), span: i32) -> bool {
    (0..span).contains(&inside.0) && (0..span).contains(&inside.1)
}

fn in_the_tunnel(inside: (i32, i32), span: i32) -> bool {
    let along_x = (0..span).contains(&inside.1) && door_band(inside.1, span);
    let along_z = (0..span).contains(&inside.0) && door_band(inside.0, span);
    along_x || along_z
}

fn door_band(value: i32, span: i32) -> bool {
    (value - span / 2).abs() <= 1
}

fn room_wall(inside: (i32, i32), span: i32) -> bool {
    let (x, z) = inside;
    let on_the_rim = x == 0 || z == 0 || x == span - 1 || z == span - 1;
    if on_the_rim {
        let gate_x = (x == 0 || x == span - 1) && door_band(z, span);
        let gate_z = (z == 0 || z == span - 1) && door_band(x, span);
        return !(gate_x || gate_z);
    }
    let line_x = x % theme::DUNGEON_ROOM == 0;
    let line_z = z % theme::DUNGEON_ROOM == 0;
    let gate_x = (3..=5).contains(&(z % theme::DUNGEON_ROOM));
    let gate_z = (3..=5).contains(&(x % theme::DUNGEON_ROOM));
    (line_x && !gate_x) || (line_z && !gate_z)
}

fn ores(world: &mut VoxelWorld, generation: &mut Generation, column: ColumnKey) {
    let veins = [
        (
            BlockKind::CoalOre,
            3,
            theme::FLOOR_Y + 4,
            theme::SEA_LEVEL + 40,
            8,
        ),
        (
            BlockKind::IronOre,
            2,
            theme::FLOOR_Y + 4,
            theme::SEA_LEVEL,
            6,
        ),
        (
            BlockKind::GoldOre,
            1,
            theme::FLOOR_Y + 4,
            theme::SEA_LEVEL - 30,
            5,
        ),
        (
            BlockKind::DiamondOre,
            1,
            theme::FLOOR_Y + 2,
            theme::FLOOR_Y + 50,
            4,
        ),
    ];
    let low = IVec3::new(
        column[0] * theme::CHUNK,
        theme::FLOOR_Y,
        column[1] * theme::CHUNK,
    );
    let high = low + IVec3::new(theme::CHUNK, theme::WORLD_TALL, theme::CHUNK);
    for dz in -1..=1 {
        for dx in -1..=1 {
            let source = [column[0] + dx, column[1] + dz];
            let mut rng =
                Random::seeded((hash(generation.seed ^ 0x00F1_2C33, source[0], source[1])) as u64);
            let start = IVec3::new(
                source[0] * theme::CHUNK,
                theme::FLOOR_Y,
                source[1] * theme::CHUNK,
            );
            for (kind, attempts, lowest, highest, size) in veins {
                for _ in 0..attempts {
                    let mut position = IVec3::new(
                        start.x + whole(&mut rng, 0, theme::CHUNK),
                        whole(&mut rng, lowest, highest),
                        start.z + whole(&mut rng, 0, theme::CHUNK),
                    );
                    for _ in 0..size {
                        let inside = position.x >= low.x
                            && position.x < high.x
                            && position.z >= low.z
                            && position.z < high.z;
                        if inside && block_at(world, position) == BlockKind::Stone {
                            write_block(world, position, kind.index());
                        }
                        position += IVec3::new(
                            whole(&mut rng, -1, 2),
                            whole(&mut rng, -1, 2),
                            whole(&mut rng, -1, 2),
                        );
                    }
                }
            }
        }
    }
}

fn spiral(radius: i32) -> impl Iterator<Item = (i32, i32)> {
    (0..=radius).flat_map(move |ring| {
        (-ring..=ring).flat_map(move |dx| {
            (-ring..=ring)
                .filter(move |dz| dx.abs() == ring || dz.abs() == ring)
                .map(move |dz| (dx, dz))
        })
    })
}

struct HeightField {
    center: (i32, i32),
    reach: i32,
    heights: Vec<i32>,
}

impl HeightField {
    fn new(noises: &Noises, plan: &WorldPlan, center: (i32, i32), reach: i32) -> Self {
        let span = (reach * 2 + 1) as usize;
        let mut heights = vec![0; span * span];
        for dz in -reach..=reach {
            for dx in -reach..=reach {
                let index = (dz + reach) as usize * span + (dx + reach) as usize;
                heights[index] = land_height(noises, plan, center.0 + dx, center.1 + dz);
            }
        }
        Self {
            center,
            reach,
            heights,
        }
    }

    fn at(&self, x: i32, z: i32) -> Option<i32> {
        let (dx, dz) = (x - self.center.0, z - self.center.1);
        if dx.abs() > self.reach || dz.abs() > self.reach {
            return None;
        }
        let span = (self.reach * 2 + 1) as usize;
        Some(self.heights[(dz + self.reach) as usize * span + (dx + self.reach) as usize])
    }

    fn dry(&self, x: i32, z: i32) -> bool {
        self.at(x, z)
            .is_some_and(|height| height > theme::SEA_LEVEL + 1)
    }
}

fn footprint_fits(
    ground: &HeightField,
    taken: &[Footprint],
    x: i32,
    z: i32,
    width: i32,
    depth: i32,
) -> Option<i32> {
    let mut lowest = i32::MAX;
    let mut highest = i32::MIN;
    for dz in -2..=depth + 1 {
        for dx in -2..=width + 1 {
            let (sx, sz) = (x + dx, z + dz);
            if !ground.dry(sx, sz) {
                return None;
            }
            if taken.iter().any(|footprint| {
                sx >= footprint.x - 3
                    && sx <= footprint.x + footprint.width + 2
                    && sz >= footprint.z - 3
                    && sz <= footprint.z + footprint.depth + 2
            }) {
                return None;
            }
            let height = ground.at(sx, sz)?;
            lowest = lowest.min(height);
            highest = highest.max(height);
        }
    }
    (highest - lowest <= theme::PAD_STEP).then_some((lowest + highest) / 2)
}

fn village_in_region(generation: &Generation, region: (i32, i32)) -> Option<Village> {
    if !generation.plan.villages {
        return None;
    }
    let mut rng = Random::seeded((hash(generation.seed ^ 0x5651_A17B, region.0, region.1)) as u64);
    if rng.u32(..) % 100 >= theme::VILLAGE_CHANCE {
        return None;
    }
    let reach = theme::VILLAGE_REACH;
    let center = (
        region.0 * theme::REGION + whole(&mut rng, reach, theme::REGION - reach),
        region.1 * theme::REGION + whole(&mut rng, reach, theme::REGION - reach),
    );
    let ground = HeightField::new(&generation.noises, &generation.plan, center, reach);
    if !ground.dry(center.0, center.1) {
        return None;
    }

    let mut village = Village {
        center,
        square: center,
        ..Default::default()
    };
    let mut doors = Vec::new();
    for (index, building) in VILLAGE.iter().enumerate() {
        let width = building.width();
        let depth = building.depth();
        let found = spiral(reach - 6).find_map(|(dx, dz)| {
            let x = center.0 + dx - width / 2;
            let z = center.1 + dz - depth / 2;
            footprint_fits(&ground, &village.buildings, x, z, width, depth).map(|base| (x, z, base))
        });
        let Some((x, z, base)) = found else {
            continue;
        };
        village.buildings.push(Footprint {
            building: index,
            x,
            z,
            width,
            depth,
            base,
        });
        let door = (x + width / 2, z + depth + 1);
        if building.name == "shop" {
            village.shop_door = Some(door);
        }
        village.torches.push((door.0 - 2, door.1));
        village.torches.push((door.0 + 2, door.1));
        doors.push(door);
    }
    if village.buildings.is_empty() {
        return None;
    }

    village.square = doors.first().copied().unwrap_or(center);
    let square = village.square;
    for door in doors.iter().skip(1) {
        pave(&mut village, *door, square);
    }

    let welcome = (square.0 + 3, square.1 + 1);
    place_sign(
        &mut village,
        &ground,
        welcome,
        format!("Welcome to {}", theme::VILLAGE_NAME),
    );
    if let Some(door) = village.shop_door {
        place_sign(
            &mut village,
            &ground,
            (door.0 + 2, door.1),
            "Bram's shop".to_string(),
        );
    }
    Some(village)
}

fn pave(village: &mut Village, from: (i32, i32), to: (i32, i32)) {
    let mut cursor = from;
    let mut steps = 0;
    while cursor != to && steps < 400 {
        steps += 1;
        if cursor.0 != to.0 {
            cursor.0 += (to.0 - cursor.0).signum();
        } else {
            cursor.1 += (to.1 - cursor.1).signum();
        }
        village.roads.push(cursor);
    }
}

fn place_sign(village: &mut Village, ground: &HeightField, near: (i32, i32), text: String) {
    let inside = |village: &Village, x: i32, z: i32| {
        village.buildings.iter().any(|footprint| {
            x >= footprint.x - 2
                && x <= footprint.x + footprint.width + 1
                && z >= footprint.z - 2
                && z <= footprint.z + footprint.depth + 1
        }) || village.roads.contains(&(x, z))
    };
    let found = spiral(6).find_map(|(dx, dz)| {
        let (x, z) = (near.0 + dx, near.1 + dz);
        if !ground.dry(x, z) || inside(village, x, z) {
            return None;
        }
        ground.at(x, z).map(|height| [x, height, z])
    });
    if let Some(block) = found {
        village.signs.push((block, text));
    }
}

fn village(world: &mut VoxelWorld, generation: &mut Generation, column: ColumnKey) {
    let origin = IVec3::new(column[0] * theme::CHUNK, 0, column[1] * theme::CHUNK);
    let low = (origin.x, origin.z);
    let high = (origin.x + theme::CHUNK - 1, origin.z + theme::CHUNK - 1);
    let inside = |x: i32, z: i32| x >= low.0 && x <= high.0 && z >= low.1 && z <= high.1;
    let mut writes: Vec<(IVec3, BlockKind)> = Vec::new();
    for village in villages_near(generation, origin.x, origin.z) {
        for footprint in &village.buildings {
            let building: &Building = &VILLAGE[footprint.building];
            for (layer_index, layer) in building.layers.iter().enumerate() {
                for (row_index, row) in layer.iter().enumerate() {
                    for (column_index, character) in row.chars().enumerate() {
                        let Some(kind) = building_block(character) else {
                            continue;
                        };
                        let position = IVec3::new(
                            footprint.x + column_index as i32,
                            footprint.base + layer_index as i32,
                            footprint.z + row_index as i32,
                        );
                        if inside(position.x, position.z) {
                            writes.push((position, kind));
                        }
                    }
                }
            }
        }
        for (x, z) in &village.roads {
            if inside(*x, *z) {
                writes.push((IVec3::new(*x, 0, *z), BlockKind::Gravel));
            }
        }
        for (block, _) in &village.signs {
            if inside(block[0], block[2]) {
                writes.push((
                    IVec3::new(block[0], block[1], block[2]),
                    BlockKind::Signpost,
                ));
            }
        }
        for (x, z) in &village.torches {
            if inside(*x, *z) {
                writes.push((IVec3::new(*x, 0, *z), BlockKind::Torch));
            }
        }
    }
    for (position, kind) in writes {
        if kind == BlockKind::Torch {
            let Some(ground) = ground_top(generation, position.x, position.z) else {
                continue;
            };
            let stand = IVec3::new(position.x, ground + 1, position.z);
            if block_at(world, stand) == BlockKind::Air
                && block_at(world, stand - IVec3::new(0, 1, 0)).info().solid
            {
                write_block(world, stand, BlockKind::Torch.index());
            }
            continue;
        }
        if kind == BlockKind::Gravel {
            let Some(ground) = ground_top(generation, position.x, position.z) else {
                continue;
            };
            let surface = IVec3::new(position.x, ground, position.z);
            let top = block_at(world, surface);
            if matches!(
                top,
                BlockKind::Grass | BlockKind::Dirt | BlockKind::Sand | BlockKind::Gravel
            ) && block_at(world, surface + IVec3::new(0, 1, 0)) == BlockKind::Air
            {
                write_block(world, surface, BlockKind::Gravel.index());
            }
            continue;
        }
        write_block(world, position, kind.index());
    }
}

fn tree_in_cell(generation: &Generation, cell: (i32, i32)) -> Option<(i32, i32)> {
    let mut rng = Random::seeded((hash(generation.seed ^ 0x7A11_0C05, cell.0, cell.1)) as u64);
    let x = cell.0 * theme::TREE_CELL + whole(&mut rng, 0, theme::TREE_CELL);
    let z = cell.1 * theme::TREE_CELL + whole(&mut rng, 0, theme::TREE_CELL);
    let biome = biome_at(generation, x, z).info();
    let chance = (biome.trees_per_hundred
        * generation.plan.woods.scale()
        * theme::TREE_CELL as f32
        * theme::TREE_CELL as f32
        / 100.0)
        .clamp(0.0, 0.95);
    (rng.f32() < chance).then_some((x, z))
}

fn trees(world: &mut VoxelWorld, generation: &mut Generation, column: ColumnKey) {
    let origin = IVec3::new(column[0] * theme::CHUNK, 0, column[1] * theme::CHUNK);
    let first = (
        origin.x.div_euclid(theme::TREE_CELL),
        origin.z.div_euclid(theme::TREE_CELL),
    );
    let last = (
        (origin.x + theme::CHUNK - 1).div_euclid(theme::TREE_CELL),
        (origin.z + theme::CHUNK - 1).div_euclid(theme::TREE_CELL),
    );
    let mut trunks = Vec::new();
    for cell_z in first.1..=last.1 {
        for cell_x in first.0..=last.0 {
            let Some((x, z)) = tree_in_cell(generation, (cell_x, cell_z)) else {
                continue;
            };
            if x < origin.x || x >= origin.x + theme::CHUNK {
                continue;
            }
            if z < origin.z || z >= origin.z + theme::CHUNK {
                continue;
            }
            if reserved(generation, x, z) {
                continue;
            }
            let Some(height) = ground_top(generation, x, z) else {
                continue;
            };
            if height <= theme::SEA_LEVEL || height + 9 >= theme::WORLD_Y {
                continue;
            }
            let ground = block_at(world, IVec3::new(x, height, z));
            if !matches!(ground, BlockKind::Grass | BlockKind::SnowGrass) {
                continue;
            }
            let pine = biome_at(generation, x, z).info().pines;
            trunks.push((x, z, height, pine));
        }
    }
    for (x, z, height, pine) in trunks {
        let mut rng = Random::seeded((hash(generation.seed ^ 0x2B7E_0011, x, z)) as u64);
        if pine {
            grow_a_pine(world, generation, x, z, height, &mut rng);
            continue;
        }
        let trunk = whole(&mut rng, 4, 7);
        for y in 1..=trunk {
            place(
                world,
                generation,
                IVec3::new(x, height + y, z),
                BlockKind::OakLog,
                false,
            );
        }
        let crown = height + trunk;
        for dy in -2..=1 {
            let radius: i32 = if dy < 0 { 2 } else { 1 };
            for dx in -radius..=radius {
                for dz in -radius..=radius {
                    let corner = dx.abs() == radius && dz.abs() == radius;
                    if corner && (dy >= 0 || rng.u32(..).is_multiple_of(3)) {
                        continue;
                    }
                    place(
                        world,
                        generation,
                        IVec3::new(x + dx, crown + dy, z + dz),
                        BlockKind::OakLeaves,
                        true,
                    );
                }
            }
        }
        place(
            world,
            generation,
            IVec3::new(x, crown + 2, z),
            BlockKind::OakLeaves,
            true,
        );
    }
}

fn grow_a_pine(
    world: &mut VoxelWorld,
    generation: &mut Generation,
    x: i32,
    z: i32,
    height: i32,
    rng: &mut Random,
) {
    let trunk = whole(rng, 6, 10);
    for y in 1..=trunk {
        place(
            world,
            generation,
            IVec3::new(x, height + y, z),
            BlockKind::OakLog,
            false,
        );
    }
    let skirt = height + 2 + whole(rng, 0, 2);
    let mut radius: i32 = 2;
    let mut y = skirt;
    while y <= height + trunk {
        for dx in -radius..=radius {
            for dz in -radius..=radius {
                if dx.abs() + dz.abs() > radius + 1 {
                    continue;
                }
                place(
                    world,
                    generation,
                    IVec3::new(x + dx, y, z + dz),
                    BlockKind::OakLeaves,
                    true,
                );
            }
        }
        if radius > 0 && (y - skirt) % 2 == 0 {
            radius -= 1;
        }
        if radius == 0 {
            radius = 2;
        }
        y += 2;
    }
    place(
        world,
        generation,
        IVec3::new(x, height + trunk + 1, z),
        BlockKind::OakLeaves,
        true,
    );
}

fn flowers(world: &mut VoxelWorld, generation: &mut Generation, column: ColumnKey) {
    let origin = IVec3::new(column[0] * theme::CHUNK, 0, column[1] * theme::CHUNK);
    for local_z in 0..theme::CHUNK {
        for local_x in 0..theme::CHUNK {
            let (x, z) = (origin.x + local_x, origin.z + local_z);
            if reserved(generation, x, z) {
                continue;
            }
            let mut rng = Random::seeded((hash(generation.seed ^ 0x1F0E_9C55, x, z)) as u64);
            let biome = biome_at(generation, x, z).info();
            let roll = rng.f32() * 100.0;
            let flowering = roll <= biome.flowers_per_hundred;
            let bushy = !flowering && roll <= biome.flowers_per_hundred + biome.bushes_per_hundred;
            if !flowering && !bushy {
                continue;
            }
            let Some(height) = ground_top(generation, x, z) else {
                continue;
            };
            let ground = IVec3::new(x, height, z);
            let above = ground + IVec3::new(0, 1, 0);
            if block_at(world, above) != BlockKind::Air {
                continue;
            }
            let footing = block_at(world, ground);
            if flowering && footing == BlockKind::Grass {
                let flower = if rng.u32(..).is_multiple_of(2) {
                    BlockKind::RedFlower
                } else {
                    BlockKind::YellowFlower
                };
                write_block(world, above, flower.index());
            }
            if bushy
                && matches!(
                    footing,
                    BlockKind::Grass
                        | BlockKind::Sand
                        | BlockKind::RedSand
                        | BlockKind::Terracotta
                        | BlockKind::SnowGrass
                )
            {
                write_block(world, above, BlockKind::DeadBush.index());
            }
        }
    }
}
