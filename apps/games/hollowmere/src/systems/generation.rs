use crate::commands::places::town_village;
use crate::commands::random::village_name;
use crate::commands::{build_room, toast_show};
use crate::data::{
    BlockKind, Building, FARM_PLAN, FISHING_PLAN, HAMLET_PLAN, MINING_PLAN, SHOP_BUILDING,
    SHRINE_PLAN, TOWN_PLAN, block_table, building_at, building_block,
};
use crate::queries::places::{lay_out_zones, town_of, village_clear_of_zones, wall_radius_of};
use crate::queries::{
    along_segment, apron_share, biome_and_rock, biome_at, block_at, block_index, cave_field,
    chip_at, chunk_of, chunks_of, column_levels, column_of, column_settled, crypt_layout,
    dungeon_at, dungeon_chest, footprint_apron, footprint_base, footprint_flatten, ground_top,
    guess_height, hash, in_bounds, in_the_rooms, in_the_tunnel, land_height, mine_layout,
    near_building, pad_reach, region_of, reserved, rock_density, shelf_height, slab_of,
    surface_height, tree_share, village_pads, villages_near, walkable, zone_signs,
};
use crate::resources::{
    ChunkStage, ColumnKey, ColumnOrder, ColumnReport, Digging, Draft, Footprint, Generation,
    Heights, Noises, Pad, Player, Save, Saved, Toast, Village, VillageKind, VoxelWorld,
};
use crate::theme;
use Random;
use nalgebra_glm::{IVec3, Vec2};
use nightshade::prelude::*;
use nightshade_random::prelude::*;
use nightshade_render::data::config::RendererState;
use nightshade_voxels::prelude::{Dig, dig_the_world, finish_column, write_block};
use noise::NoiseFn;
use std::collections::HashMap;
use std::sync::Arc;

const PASSES: [fn(&mut VoxelWorld, &mut Generation, ColumnKey); 10] = [
    terrain,
    water,
    caves,
    dungeon,
    ores,
    carried,
    crate::systems::zones::zones,
    village,
    trees,
    flowers,
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

pub fn raise_the_new_world(
    mut draft: ResMut<Draft>,
    mut player: ResMut<Player>,
    mut world: ResMut<VoxelWorld>,
    mut generation: ResMut<Generation>,
    mut renderer_state: ResMut<RendererState>,
    mut toast: ResMut<Toast>,
) {
    if !draft.building {
        return;
    }
    draft.building = false;
    generation.seed = draft.seed;
    generation.plan = draft.plan;
    stand_the_player_up(&mut player, &mut world, &mut generation, None);
    renderer_state.day_night.hour = generation.plan.start.hour();
    toast_show(&mut toast, "A new world");
}

fn stand_the_player_up(
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
    let town = found.and_then(|found| found.town).unwrap_or_else(|| {
        load_villages_near(generation, column_of(spawn.0, spawn.1));
        town_of(generation, spawn)
    });
    load_villages_near(generation, column_of(town.0, town.1));
    lay_out_the_land(generation, town);
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
}

fn lay_out_the_land(generation: &mut Generation, town: (i32, i32)) {
    let radius = match town_village(generation, town) {
        Some(village) => {
            village.name = theme::TITLE.to_string();
            for (_, text) in village.signs.iter_mut() {
                if text.starts_with("Welcome to") {
                    *text = format!("Welcome to {}", theme::TITLE);
                }
            }
            wall_radius_of(village, town)
        }
        None => theme::TOWN_WALL_MIN as f32,
    };
    generation.zones = lay_out_zones(generation, town, radius);
    generation.crypt = crypt_layout(generation);
    generation.mine = mine_layout(generation);
    let zones = generation.zones.clone();
    for village in generation.villages.values_mut() {
        if village.as_ref().is_some_and(|village| {
            village.square != town && !village_clear_of_zones(&zones, village.center)
        }) {
            *village = None;
        }
    }
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
    let home = generation
        .villages
        .get(&(0, 0))
        .and_then(|village| village.as_ref())
        .map(|village| village.square)
        .or_else(|| {
            villages_near(generation, 0, 0)
                .min_by_key(|village| village.center.0.abs() + village.center.1.abs())
                .map(|village| village.square)
        });
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
            zones: generation.zones.clone(),
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
        if tools.zones != order.zones {
            tools.zones = order.zones;
            tools.crypt = crypt_layout(tools);
            tools.mine = mine_layout(tools);
        }
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
    let mut signs: Vec<([i32; 3], String)> = villages_near(generation, origin.x, origin.z)
        .flat_map(|village| village.signs.iter())
        .filter(|(block, _)| column_of(block[0], block[2]) == column)
        .cloned()
        .collect();
    for ((x, z), text) in zone_signs(generation) {
        if column_of(x, z) != column {
            continue;
        }
        let y =
            ground_top(generation, x, z).unwrap_or_else(|| land_height(generation, x, z) - 1) + 1;
        signs.push(([x, y, z], text));
    }
    signs
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

fn column_height(generation: &Generation, x: i32, z: i32) -> i32 {
    if let Some(heights) = &generation.heights {
        let span = theme::CHUNK + 2;
        let offset_x = x - heights.column[0] * theme::CHUNK + 1;
        let offset_z = z - heights.column[1] * theme::CHUNK + 1;
        if (0..span).contains(&offset_x) && (0..span).contains(&offset_z) {
            return heights.values[(offset_z * span + offset_x) as usize];
        }
    }
    guess_height(generation, x, z)
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
            lattice.push(land_height(generation, x, z) as f32);
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
            let natural =
                lattice_height(&lattice, nodes, step, offset_x, offset_z).round() as i32 - 1;
            values.push(match footprint_apron(generation, x, z) {
                Some((base, share)) => {
                    (natural as f32 + (base - natural) as f32 * share).round() as i32
                }
                None => natural,
            });
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
                    let mut value = rocky * (1.0 - flatten) + (plane + 0.5 - y as f32) * flatten;
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
    fn new(generation: &Generation, center: (i32, i32), reach: i32) -> Self {
        let span = (reach * 2 + 1) as usize;
        let mut heights = vec![0; span * span];
        for dz in -reach..=reach {
            for dx in -reach..=reach {
                let index = (dz + reach) as usize * span + (dx + reach) as usize;
                heights[index] = land_height(generation, center.0 + dx, center.1 + dz);
            }
        }
        Self {
            center,
            reach,
            heights,
        }
    }

    fn level(center: (i32, i32), reach: i32, level: i32) -> Self {
        let span = (reach * 2 + 1) as usize;
        Self {
            center,
            reach,
            heights: vec![level; span * span],
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

const ARMS: [(i32, i32); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];

struct Site<'a> {
    center: (i32, i32),
    ground: &'a HeightField,
    home: bool,
    spread: i32,
}

struct Row {
    arm: usize,
    side: i32,
    cursor: i32,
}

struct Lot {
    x: i32,
    z: i32,
    width: i32,
    depth: i32,
    facing: u8,
    front: (i32, i32),
    door: Option<(i32, i32)>,
}

fn arm_cell(center: (i32, i32), arm: usize, along: i32, across: i32) -> (i32, i32) {
    let (axis_x, axis_z) = ARMS[arm];
    if axis_x != 0 {
        (center.0 + axis_x * along, center.1 + across)
    } else {
        (center.0 + across, center.1 + axis_z * along)
    }
}

fn turn(column: i32, row: i32, width: i32, depth: i32, facing: u8) -> (i32, i32) {
    match facing {
        1 => (depth - 1 - row, column),
        2 => (width - 1 - column, depth - 1 - row),
        3 => (row, width - 1 - column),
        _ => (column, row),
    }
}

fn outward(facing: u8) -> (i32, i32) {
    match facing {
        1 => (-1, 0),
        2 => (0, -1),
        3 => (1, 0),
        _ => (0, 1),
    }
}

fn turned(kind: BlockKind, facing: u8) -> BlockKind {
    if kind != BlockKind::DoorSouth {
        return kind;
    }
    match facing {
        1 => BlockKind::DoorWest,
        2 => BlockKind::DoorNorth,
        3 => BlockKind::DoorEast,
        _ => kind,
    }
}

fn lot_for(center: (i32, i32), arm: usize, side: i32, along: i32, building: &Building) -> Lot {
    let (width, depth) = (building.width(), building.depth());
    let near = theme::STREET_HALF + 2;
    let first = arm_cell(center, arm, along, side * near);
    let last = arm_cell(center, arm, along + width - 1, side * (near + depth - 1));
    let x = first.0.min(last.0);
    let z = first.1.min(last.1);
    let facing = match (ARMS[arm].0 != 0, side < 0) {
        (true, true) => 0,
        (true, false) => 2,
        (false, true) => 3,
        (false, false) => 1,
    };
    let door = building.doorway().map(|column| {
        let (door_x, door_z) = turn(column, depth - 1, width, depth, facing);
        let (out_x, out_z) = outward(facing);
        (x + door_x + out_x, z + door_z + out_z)
    });
    Lot {
        x,
        z,
        width: (first.0 - last.0).abs() + 1,
        depth: (first.1 - last.1).abs() + 1,
        facing,
        front: arm_cell(
            center,
            arm,
            along + width / 2,
            side * (theme::STREET_HALF + 1),
        ),
        door,
    }
}

fn planned_top(site: &Site, village: &Village, x: i32, z: i32) -> i32 {
    let natural = site.ground.at(x, z).unwrap_or(village.level + 1) - 1;
    let mut best: Option<(i32, f32)> = None;
    for pad in village_pads(village) {
        let share = apron_share(pad_reach(&pad, x, z));
        if share > 0.0 && best.is_none_or(|(_, held)| share > held) {
            best = Some((pad.base, share));
        }
    }
    match best {
        Some((base, share)) => (natural as f32 + (base - natural) as f32 * share).round() as i32,
        None => natural,
    }
}

fn lot_fits(site: &Site, village: &Village, lot: &Lot) -> Option<i32> {
    for z in lot.z - 1..=lot.z + lot.depth {
        for x in lot.x - 1..=lot.x + lot.width {
            if !site.ground.dry(x, z) {
                return None;
            }
        }
    }
    if site.home {
        for (x, z) in [
            (lot.x, lot.z),
            (lot.x + lot.width, lot.z),
            (lot.x, lot.z + lot.depth),
            (lot.x + lot.width, lot.z + lot.depth),
        ] {
            let reach =
                Vec2::new((x - site.center.0) as f32, (z - site.center.1) as f32).magnitude();
            if reach > site.spread as f32 {
                return None;
            }
        }
    }
    let clear = |pad: &Pad, gap: i32| {
        lot.x + lot.width - 1 + gap < pad.x
            || lot.x - gap > pad.x + pad.width - 1
            || lot.z + lot.depth - 1 + gap < pad.z
            || lot.z - gap > pad.z + pad.depth - 1
    };
    let mut pads = village_pads(village);
    if let Some(plaza) = pads.next()
        && !clear(&plaza, 1)
    {
        return None;
    }
    if pads.any(|pad| !clear(&pad, theme::LOT_GAP)) {
        return None;
    }
    let (front_x, front_z) = lot.door.unwrap_or(lot.front);
    let base = planned_top(site, village, front_x, front_z);
    for z in lot.z..lot.z + lot.depth {
        for x in lot.x..lot.x + lot.width {
            let top = site.ground.at(x, z)? - 1;
            if (top - base).abs() > theme::TERRACE_MAX {
                return None;
            }
        }
    }
    Some(base)
}

fn arm_limits(site: &Site) -> [i32; 4] {
    let mut limits = [theme::PLAZA_HALF; 4];
    for (arm, limit) in limits.iter_mut().enumerate() {
        for along in theme::PLAZA_HALF + 1..=site.spread {
            let wet = (-theme::STREET_HALF..=theme::STREET_HALF).any(|across| {
                let (x, z) = arm_cell(site.center, arm, along, across);
                !site.ground.dry(x, z)
            });
            if wet {
                break;
            }
            *limit = along;
        }
    }
    limits
}

fn stand_on_the_plaza(village: &mut Village, spot: (i32, i32), index: usize) {
    let building = building_at(index);
    let (width, depth) = (building.width(), building.depth());
    village.buildings.push(Footprint {
        building: index,
        x: spot.0 - width / 2,
        z: spot.1 - depth / 2,
        width,
        depth,
        base: village.level,
        facing: 0,
    });
}

fn place_on_a_street(
    site: &Site,
    village: &mut Village,
    rows: &mut [Row],
    limits: &[i32; 4],
    far: &mut [i32; 4],
    index: usize,
) {
    let building = building_at(index);
    let width = building.width();
    loop {
        rows.sort_by_key(|row| (row.cursor, row.arm, row.side));
        let Some(row) = rows
            .iter_mut()
            .find(|row| row.cursor + width - 1 <= limits[row.arm])
        else {
            return;
        };
        let lot = lot_for(site.center, row.arm, row.side, row.cursor, building);
        let Some(base) = lot_fits(site, village, &lot) else {
            row.cursor += 1;
            continue;
        };
        let end = row.cursor + width - 1;
        far[row.arm] = far[row.arm].max(end);
        row.cursor = end + 1 + theme::LOT_GAP;
        village.buildings.push(Footprint {
            building: index,
            x: lot.x,
            z: lot.z,
            width: lot.width,
            depth: lot.depth,
            base,
            facing: lot.facing,
        });
        if let Some(door) = lot.door {
            let (axis_x, axis_z) = ARMS[row.arm];
            village.roads.push(door);
            village
                .torches
                .push((door.0 + axis_x * 2, door.1 + axis_z * 2));
            village
                .torches
                .push((door.0 - axis_x * 2, door.1 - axis_z * 2));
            if index == SHOP_BUILDING {
                village.shop_door = Some(door);
            }
        }
        return;
    }
}

fn pick_site(
    generation: &Generation,
    rng: &mut Random,
    region: (i32, i32),
    reach: i32,
    home: bool,
) -> Option<(i32, i32)> {
    let mut best: Option<((i32, i32), i32)> = None;
    for _ in 0..theme::SITE_TRIES {
        let center = (
            region.0 * theme::REGION + whole(rng, reach, theme::REGION - reach),
            region.1 * theme::REGION + whole(rng, reach, theme::REGION - reach),
        );
        let mut low = i32::MAX;
        let mut high = i32::MIN;
        for dz in (-theme::SITE_RADIUS..=theme::SITE_RADIUS).step_by(3) {
            for dx in (-theme::SITE_RADIUS..=theme::SITE_RADIUS).step_by(3) {
                let height = land_height(generation, center.0 + dx, center.1 + dz);
                low = low.min(height);
                high = high.max(height);
            }
        }
        let wet = low <= theme::SEA_LEVEL + 1;
        if wet && !home {
            continue;
        }
        let score = high - low + if wet { 100 } else { 0 };
        if best.is_none_or(|(_, held)| score < held) {
            best = Some((center, score));
        }
    }
    let (center, score) = best?;
    (home || score <= theme::SITE_RANGE_MAX).then_some(center)
}

fn village_in_region(generation: &Generation, region: (i32, i32)) -> Option<Village> {
    if !generation.plan.villages {
        return None;
    }
    let mut rng = Random::seeded((hash(generation.seed ^ 0x5651_A17B, region.0, region.1)) as u64);
    let home = region == (0, 0);
    if !home && rng.u32(..) % 100 >= theme::VILLAGE_CHANCE {
        return None;
    }
    let roll = rng.u32(..) % 100;
    let mut kind = if home || roll < theme::TOWN_SHARE {
        VillageKind::Town
    } else if roll < theme::TOWN_SHARE + theme::FARM_SHARE {
        VillageKind::Farm
    } else if roll < theme::TOWN_SHARE + theme::FARM_SHARE + theme::SHRINE_SHARE {
        VillageKind::Shrine
    } else if roll
        < theme::TOWN_SHARE + theme::FARM_SHARE + theme::SHRINE_SHARE + theme::MINING_SHARE
    {
        VillageKind::Mining
    } else {
        VillageKind::Fishing
    };
    let name = village_name(&mut rng);
    let reach = if home {
        theme::TOWN_REACH
    } else {
        theme::VILLAGE_REACH
    };
    let center = pick_site(generation, &mut rng, region, reach, home)?;
    if !village_clear_of_zones(&generation.zones, center) {
        return None;
    }
    let standing = land_height(generation, center.0, center.1);
    let level = if home {
        standing.max(theme::TOWN_LEVEL_MIN)
    } else {
        standing
    };
    let ground = if home {
        HeightField::level(center, reach, level)
    } else {
        HeightField::new(generation, center, reach)
    };
    let pier = pier_from(&ground, center);
    if kind == VillageKind::Fishing && pier.is_none() {
        kind = VillageKind::Hamlet;
    }
    if kind == VillageKind::Mining
        && !matches!(
            biome_at(generation, center.0, center.1),
            crate::data::Biome::Hills | crate::data::Biome::Peaks | crate::data::Biome::Badlands
        )
    {
        kind = VillageKind::Hamlet;
    }
    let plan: &[usize] = match kind {
        VillageKind::Town => &TOWN_PLAN,
        VillageKind::Farm => &FARM_PLAN,
        VillageKind::Fishing => &FISHING_PLAN,
        VillageKind::Mining => &MINING_PLAN,
        VillageKind::Shrine => &SHRINE_PLAN,
        VillageKind::Hamlet => &HAMLET_PLAN,
    };

    let mut village = Village {
        center,
        square: center,
        kind,
        name,
        level: level - 1,
        pier: if kind == VillageKind::Fishing {
            pier
        } else {
            None
        },
        ..Default::default()
    };
    let site = Site {
        center,
        ground: &ground,
        home,
        spread: if home {
            theme::TOWN_SPREAD
        } else {
            theme::VILLAGE_SPREAD
        },
    };
    let half = theme::PLAZA_HALF;
    for dz in -half..=half {
        for dx in -half..=half {
            village.roads.push((center.0 + dx, center.1 + dz));
        }
    }
    let (first, rest) = plan.split_first()?;
    stand_on_the_plaza(&mut village, center, *first);
    let step = half - 1;
    let mut slots = [
        (center.0 - step, center.1 - step),
        (center.0 + step, center.1 - step),
        (center.0 - step, center.1 + step),
        (center.0 + step, center.1 + step),
    ]
    .into_iter();
    let limits = arm_limits(&site);
    let mut far = [0i32; 4];
    let mut rows: Vec<Row> = (0..4)
        .flat_map(|arm| {
            [-1, 1].map(|side| Row {
                arm,
                side,
                cursor: half + 2,
            })
        })
        .collect();
    for index in rest.iter().copied() {
        let building = building_at(index);
        if building.width() * building.depth() <= theme::PLAZA_ITEM
            && let Some(spot) = slots.next()
        {
            stand_on_the_plaza(&mut village, spot, index);
            continue;
        }
        place_on_a_street(&site, &mut village, &mut rows, &limits, &mut far, index);
    }
    for arm in 0..4 {
        if far[arm] == 0 {
            continue;
        }
        let length = (far[arm] + 2).min(limits[arm]);
        village.streets[arm] = length;
        for along in half + 1..=length {
            for across in -theme::STREET_HALF..=theme::STREET_HALF {
                village.roads.push(arm_cell(center, arm, along, across));
            }
        }
    }

    let welcome = (center.0 + half + 2, center.1 + half + 2);
    let kind_name = match village.kind {
        VillageKind::Town => "town",
        VillageKind::Farm => "farmstead",
        VillageKind::Fishing => "fishing village",
        VillageKind::Mining => "mining camp",
        VillageKind::Shrine => "shrine",
        VillageKind::Hamlet => "hamlet",
    };
    let welcome_text = format!("Welcome to {}, a {kind_name}", village.name);
    place_sign(&site, &mut village, welcome, welcome_text);
    if let Some(door) = village.shop_door {
        place_sign(
            &site,
            &mut village,
            (door.0 + 3, door.1 + 3),
            "Trading post".to_string(),
        );
    }
    Some(village)
}

fn pier_from(ground: &HeightField, center: (i32, i32)) -> Option<(Vec2, Vec2)> {
    for step in 0..16 {
        let angle = step as f32 / 16.0 * std::f32::consts::TAU;
        let (sine, cosine) = angle.sin_cos();
        for radius in 8..=22 {
            let x = center.0 + (cosine * radius as f32).round() as i32;
            let z = center.1 + (sine * radius as f32).round() as i32;
            let Some(height) = ground.at(x, z) else {
                break;
            };
            if height > theme::SEA_LEVEL - 1 {
                continue;
            }
            let shore = radius as f32 - 2.0;
            let start = Vec2::new(
                center.0 as f32 + 0.5 + cosine * shore,
                center.1 as f32 + 0.5 + sine * shore,
            );
            let end = start + Vec2::new(cosine, sine) * theme::PIER_RUN;
            return Some((start, end));
        }
    }
    None
}

fn place_sign(site: &Site, village: &mut Village, near: (i32, i32), text: String) {
    let found = spiral(6).find_map(|(dx, dz)| {
        let (x, z) = (near.0 + dx, near.1 + dz);
        if !site.ground.dry(x, z) || village.roads.contains(&(x, z)) {
            return None;
        }
        if village_pads(village).any(|pad| pad_reach(&pad, x, z) == 0) {
            return None;
        }
        Some([x, planned_top(site, village, x, z) + 1, z])
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
            let building: &Building = building_at(footprint.building);
            let (width, depth) = (building.width(), building.depth());
            for (layer_index, layer) in building.layers.iter().enumerate() {
                for (row_index, row) in layer.iter().enumerate() {
                    for (column_index, character) in row.chars().enumerate() {
                        let Some(kind) = building_block(character) else {
                            continue;
                        };
                        let (dx, dz) = turn(
                            column_index as i32,
                            row_index as i32,
                            width,
                            depth,
                            footprint.facing,
                        );
                        let position = IVec3::new(
                            footprint.x + dx,
                            footprint.base + layer_index as i32 - building.sink,
                            footprint.z + dz,
                        );
                        if inside(position.x, position.z) {
                            writes.push((position, turned(kind, footprint.facing)));
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
            if !inside(*x, *z) {
                continue;
            }
            let Some(ground) = ground_top(generation, *x, *z) else {
                continue;
            };
            let stand = IVec3::new(*x, ground + 1, *z);
            if block_at(world, stand) == BlockKind::Air
                && block_at(world, stand - IVec3::new(0, 1, 0)).info().solid
            {
                write_block(world, stand, BlockKind::Torch.index());
            }
        }
        if let Some((start, end)) = village.pier {
            stamp_pier(world, start, end, low, high);
        }
    }
    for (position, kind) in writes {
        if kind == BlockKind::Gravel && position.y == 0 {
            let Some(ground) = ground_top(generation, position.x, position.z) else {
                continue;
            };
            let surface = IVec3::new(position.x, ground, position.z);
            let top = block_at(world, surface);
            if matches!(
                top,
                BlockKind::Grass
                    | BlockKind::Dirt
                    | BlockKind::Sand
                    | BlockKind::Gravel
                    | BlockKind::SnowGrass
            ) && block_at(world, surface + IVec3::new(0, 1, 0)) == BlockKind::Air
            {
                write_block(world, surface, BlockKind::Gravel.index());
            }
            continue;
        }
        write_block(world, position, kind.index());
    }
}

fn stamp_pier(world: &mut VoxelWorld, start: Vec2, end: Vec2, low: (i32, i32), high: (i32, i32)) {
    let deck = theme::SEA_LEVEL + 1;
    for z in low.1..=high.1 {
        for x in low.0..=high.0 {
            let point = Vec2::new(x as f32 + 0.5, z as f32 + 0.5);
            let (along, perpendicular) = along_segment(point, start, end);
            if !(0.0..=theme::PIER_RUN).contains(&along) {
                continue;
            }
            if perpendicular <= 1.0 {
                for y in deck + 1..=deck + 4 {
                    write_block(world, IVec3::new(x, y, z), BlockKind::Air.index());
                }
                write_block(world, IVec3::new(x, deck, z), BlockKind::OakPlanks.index());
            } else if perpendicular <= 1.6 && along.rem_euclid(3.0) < 1.0 {
                for y in deck - 3..=deck + 1 {
                    write_block(world, IVec3::new(x, y, z), BlockKind::OakLog.index());
                }
                if along.rem_euclid(6.0) < 1.0 {
                    write_block(world, IVec3::new(x, deck + 2, z), BlockKind::Torch.index());
                }
            }
        }
    }
}

fn tree_in_cell(generation: &Generation, cell: (i32, i32)) -> Option<(i32, i32)> {
    let mut rng = Random::seeded((hash(generation.seed ^ 0x7A11_0C05, cell.0, cell.1)) as u64);
    let x = cell.0 * theme::TREE_CELL + whole(&mut rng, 0, theme::TREE_CELL);
    let z = cell.1 * theme::TREE_CELL + whole(&mut rng, 0, theme::TREE_CELL);
    let biome = biome_at(generation, x, z).info();
    let chance = (biome.trees_per_hundred
        * generation.plan.woods.scale()
        * tree_share(generation, x, z)
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
