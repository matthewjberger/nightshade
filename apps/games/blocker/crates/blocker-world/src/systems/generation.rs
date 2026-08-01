use crate::data::world::biomes::Biome;
use crate::data::world::nodes::{Density, NodeKind, WorkKind};
use crate::queries::{
    biome_for, cell_anchor, ground_is_buildable, ground_is_claimed, hash, surface_height, vein_at,
};
use crate::resources::{Bench, ColumnReport, Digging, Generation, WorldOrder};
use crate::theme;
use blocker_blocks::data::blocks::BlockKind;
use blocker_blocks::queries::{block_index, chunks_of};
use blocker_blocks::resources::{ChunkStage, ColumnKey, VoxelWorld};
use nalgebra_glm::IVec3;
use nightshade::prelude::*;
use nightshade_voxels::prelude::Dig;
use std::sync::Arc;
mod dens;
mod layout;
mod stamps;

use self::dens::{stamp_the_barrows, stamp_the_camps, stamp_the_landmarks, stamp_the_pois};
use self::layout::lay_out_the_village;
use self::stamps::{finish_the_stamps, queue_stamp};

impl Dig for Digging {
    type Orders = Generation;
    type Order = WorldOrder;
    type Tools = Bench;
    type Report = ColumnReport;

    fn order(orders: &mut Generation, _column: ColumnKey) -> WorldOrder {
        WorldOrder {
            seed: orders.seed,
            enabled: orders.enabled,
        }
    }

    fn dig(
        order: WorldOrder,
        tools: &mut Bench,
        world: &mut VoxelWorld,
        column: ColumnKey,
    ) -> ColumnReport {
        if tools.seed != order.seed {
            tools.reseed(order.seed);
        }
        dig_the_column(world, tools, order.seed, order.enabled, column)
    }

    fn settle(
        orders: &mut Generation,
        _world: &mut VoxelWorld,
        _column: ColumnKey,
        report: ColumnReport,
    ) -> Vec<ColumnKey> {
        for (block, text) in report.signs {
            orders.signs.insert(block, text);
        }
        for (block, text) in report.plaques {
            orders.plaques.insert(block, text);
        }
        for (village, door) in report.homes {
            let doors = orders.homes.entry(village).or_default();
            if !doors.contains(&door) {
                doors.push(door);
            }
        }
        for (village, door) in report.guides {
            orders.guides.insert(village, door);
        }
        for (village, post, kind) in report.posts {
            let posts = orders.posts.entry(village).or_default();
            if !posts.iter().any(|(place, _)| *place == post) {
                posts.push((post, kind));
            }
        }
        Vec::new()
    }
}

struct Site<'world> {
    world: &'world mut VoxelWorld,
    bench: &'world Bench,
    column: ColumnKey,
    seed: u32,
    enabled: u32,
    signs: Vec<([i32; 3], String)>,
    plaques: Vec<([i32; 3], String)>,
    homes: Vec<([i32; 2], [i32; 3])>,
    guides: Vec<([i32; 2], [i32; 3])>,
    posts: Vec<([i32; 2], [i32; 3], WorkKind)>,
    stamps: Vec<Stamping>,
    walls: Vec<Rampart>,
    markers: Vec<(IVec3, BlockKind)>,
}

struct Stamping {
    schematic: usize,
    origin: IVec3,
    rot: u8,
    paved: bool,
    held: bool,
}

struct Rampart {
    center: IVec3,
    reach: i32,
    path: BlockKind,
}

struct Instance {
    node: usize,
    origin: IVec3,
    rot: u8,
    seed: u32,
    paved: bool,
    held: bool,
}

fn dig_the_column(
    world: &mut VoxelWorld,
    bench: &Bench,
    seed: u32,
    enabled: u32,
    column: ColumnKey,
) -> ColumnReport {
    let mut site = Site {
        world,
        bench,
        column,
        seed,
        enabled,
        signs: Vec::new(),
        plaques: Vec::new(),
        homes: Vec::new(),
        guides: Vec::new(),
        posts: Vec::new(),
        stamps: Vec::new(),
        walls: Vec::new(),
        markers: Vec::new(),
    };
    let mut queue: Vec<Instance> = Vec::new();
    let origin_x = column[0] * nightshade_voxels::data::CHUNK;
    let origin_z = column[1] * nightshade_voxels::data::CHUNK;
    let reach = crate::data::world::nodes::CAPITAL_REACH;
    for seat in bench.seats.iter() {
        let at = seat.city;
        if at[0] + reach < origin_x
            || at[0] - reach > origin_x + nightshade_voxels::data::CHUNK - 1
            || at[1] + reach < origin_z
            || at[1] - reach > origin_z + nightshade_voxels::data::CHUNK - 1
        {
            continue;
        }
        let height = surface_height(bench, at[0], at[1]);
        queue.push(Instance {
            node: crate::data::world::nodes::CAPITAL,
            origin: IVec3::new(at[0], height, at[1]),
            rot: 0,
            seed: seed ^ 0xCA91_7A10,
            paved: false,
            held: true,
        });
    }
    queue.push(Instance {
        node: crate::data::world::nodes::ROOT,
        origin: IVec3::zeros(),
        rot: 0,
        seed,
        paved: false,
        held: false,
    });
    while let Some(instance) = queue.pop() {
        expand(&mut site, &mut queue, instance);
    }
    stamp_the_landmarks(&mut site);
    stamp_the_pois(&mut site);
    stamp_the_barrows(&mut site);
    stamp_the_camps(&mut site);
    finish_the_stamps(&mut site);
    ColumnReport {
        signs: site.signs,
        plaques: site.plaques,
        homes: site.homes,
        guides: site.guides,
        posts: site.posts,
    }
}

fn expand(site: &mut Site, queue: &mut Vec<Instance>, instance: Instance) {
    match &crate::data::world::nodes::NODES[instance.node].kind {
        NodeKind::Base { children } => {
            paint_the_base(site);
            for &child in *children {
                queue.push(Instance {
                    node: child,
                    origin: IVec3::zeros(),
                    rot: 0,
                    seed: site.seed ^ (child as u32).wrapping_mul(0x9E37_79B1),
                    paved: false,
                    held: false,
                });
            }
        }
        NodeKind::Scatter {
            cell,
            density,
            child,
            reach,
            feature,
        } => scatter(site, queue, *cell, density, *child, *reach, *feature),
        NodeKind::Pick { options } => {
            let total: u32 = options
                .iter()
                .map(|(weight, _)| *weight)
                .sum::<u32>()
                .max(1);
            let mut roll = instance.seed % total;
            for (weight, node) in *options {
                if roll < *weight {
                    queue.push(Instance {
                        node: *node,
                        origin: instance.origin,
                        rot: instance.rot,
                        seed: instance.seed.rotate_left(9),
                        paved: instance.paved,
                        held: instance.held,
                    });
                    break;
                }
                roll -= *weight;
            }
        }
        NodeKind::Grove { leafy, conifer } => {
            let biome = biome_for(
                site.bench,
                instance.origin.x,
                instance.origin.z,
                instance.origin.y,
            );
            let set = if matches!(
                biome,
                Biome::Taiga
                    | Biome::Tundra
                    | Biome::SnowyPlains
                    | Biome::SnowyPeaks
                    | Biome::Mountain
            ) {
                *conifer
            } else {
                *leafy
            };
            if !set.is_empty() {
                let choice = set[instance.seed as usize % set.len()];
                queue.push(Instance {
                    node: choice,
                    origin: instance.origin,
                    rot: instance.rot,
                    seed: instance.seed.rotate_left(7),
                    paved: instance.paved,
                    held: instance.held,
                });
            }
        }
        NodeKind::Layout(layout) => lay_out_the_village(site, queue, &instance, layout),
        NodeKind::Stamp { schematic } => {
            queue_stamp(
                site,
                *schematic,
                instance.origin,
                instance.rot,
                instance.paved,
                instance.held,
            );
        }
    }
}

fn scatter(
    site: &mut Site,
    queue: &mut Vec<Instance>,
    cell: i32,
    density: &Density,
    child: usize,
    reach: i32,
    feature: usize,
) {
    if site.enabled & (1u32 << feature) == 0 {
        return;
    }
    let origin_x = site.column[0] * nightshade_voxels::data::CHUNK;
    let origin_z = site.column[1] * nightshade_voxels::data::CHUNK;
    let margin = if reach * 4 < cell { reach } else { 0 };
    let low_x = (origin_x - reach).div_euclid(cell);
    let high_x = (origin_x + nightshade_voxels::data::CHUNK - 1 + reach).div_euclid(cell);
    let low_z = (origin_z - reach).div_euclid(cell);
    let high_z = (origin_z + nightshade_voxels::data::CHUNK - 1 + reach).div_euclid(cell);
    for grid_z in low_z..=high_z {
        for grid_x in low_x..=high_x {
            let (anchor_x, anchor_z) =
                cell_anchor(site.seed, child as u32, cell, grid_x, grid_z, margin);
            let height = surface_height(site.bench, anchor_x, anchor_z);
            if matches!(density, Density::Settlement)
                && !ground_is_buildable(site.bench, anchor_x, anchor_z, theme::SETTLEMENT_FOOTPRINT)
            {
                continue;
            }
            if matches!(density, Density::Ocean)
                && height > crate::data::world::SEA_LEVEL - theme::SHIP_DEPTH
            {
                continue;
            }
            if !matches!(density, Density::Ocean) && height <= crate::data::world::SEA_LEVEL {
                continue;
            }
            if matches!(density, Density::Flat(_))
                && !ground_is_buildable(site.bench, anchor_x, anchor_z, reach)
            {
                continue;
            }
            if !matches!(density, Density::Ocean)
                && ground_is_claimed(site.bench, site.seed, anchor_x, anchor_z, reach)
            {
                continue;
            }
            let biome = biome_for(site.bench, anchor_x, anchor_z, height).info();
            let chance = match density {
                Density::Flat(value) => *value,
                Density::Tree => biome.tree_chance,
                Density::Flower => biome.flower_chance,
                Density::Brush => biome.brush_chance,
                Density::Settlement => biome.village_weight,
                Density::Ocean => theme::SHIP_CHANCE,
            };
            if hash(
                site.seed ^ (child as u32).wrapping_add(0x51ED),
                grid_x,
                grid_z,
            ) % 100
                >= chance
            {
                continue;
            }
            if anchor_x + reach < origin_x
                || anchor_x - reach > origin_x + nightshade_voxels::data::CHUNK - 1
                || anchor_z + reach < origin_z
                || anchor_z - reach > origin_z + nightshade_voxels::data::CHUNK - 1
            {
                continue;
            }
            let base_y = if matches!(density, Density::Ocean) {
                crate::data::world::SEA_LEVEL
            } else {
                height
            };
            queue.push(Instance {
                node: child,
                origin: IVec3::new(anchor_x, base_y, anchor_z),
                rot: (hash(site.seed ^ 0x2000_0001, grid_x, grid_z) % 4) as u8,
                paved: false,
                held: false,
                seed: hash(
                    site.seed ^ (child as u32).wrapping_add(0x30D1),
                    grid_x,
                    grid_z,
                ),
            });
        }
    }
}

fn ground_at(bench: &Bench, x: i32, z: i32) -> i32 {
    surface_height(bench, x, z).max(crate::data::world::SEA_LEVEL + 1)
}

fn paint_the_base(site: &mut Site) {
    for key in chunks_of(site.column) {
        let chunk = site.world.chunks.entry(key).or_default();
        if chunk.stage < ChunkStage::Terrain {
            chunk.stage = ChunkStage::Terrain;
        }
    }
    let origin_x = site.column[0] * nightshade_voxels::data::CHUNK;
    let origin_z = site.column[1] * nightshade_voxels::data::CHUNK;
    for local_z in 0..nightshade_voxels::data::CHUNK {
        for local_x in 0..nightshade_voxels::data::CHUNK {
            let x = origin_x + local_x;
            let z = origin_z + local_z;
            let height = surface_height(site.bench, x, z);
            let biome = biome_for(site.bench, x, z, height).info();
            let surface = biome.surface;
            let mut vein_left = 0i32;
            let mut vein_kind = BlockKind::Stone;
            for y in nightshade_voxels::data::FLOOR_Y..=height {
                let kind = if y == nightshade_voxels::data::FLOOR_Y {
                    BlockKind::Bedrock
                } else if y == height {
                    surface
                } else if y >= height - 3 {
                    biome.filler
                } else {
                    if vein_left > 0 {
                        vein_left -= 1;
                    } else if let Some(vein) = vein_at(site.seed, x, y, z) {
                        vein_left = vein.run - 1;
                        vein_kind = vein.kind;
                    } else {
                        vein_kind = BlockKind::Stone;
                    }
                    vein_kind
                };
                set(site.world, IVec3::new(x, y, z), kind, site.column);
            }
            for y in (height + 1)..=crate::data::world::SEA_LEVEL {
                set(
                    site.world,
                    IVec3::new(x, y, z),
                    BlockKind::Water,
                    site.column,
                );
            }
        }
    }
}

fn owns(column: ColumnKey, position: IVec3) -> bool {
    position.x.div_euclid(nightshade_voxels::data::CHUNK) == column[0]
        && position.z.div_euclid(nightshade_voxels::data::CHUNK) == column[1]
}

fn set(world: &mut VoxelWorld, position: IVec3, kind: BlockKind, column: ColumnKey) {
    if !owns(column, position)
        || position.y < nightshade_voxels::data::FLOOR_Y
        || position.y >= nightshade_voxels::data::WORLD_Y
    {
        return;
    }
    let level = position.y.div_euclid(nightshade_voxels::data::CHUNK);
    let key = [column[0], level, column[1]];
    let chunk = world.chunks.entry(key).or_default();
    if chunk.stage < ChunkStage::Terrain {
        chunk.stage = ChunkStage::Terrain;
    }
    let index = block_index(position);
    let previous = chunk.blocks[index];
    Arc::make_mut(&mut chunk.blocks)[index] = kind.index();
    let cell = [position.x, position.y, position.z];
    if world.blocks.look(kind.index()).marked {
        world
            .markers
            .entry(key)
            .or_default()
            .insert(cell, kind.index());
    } else if world.blocks.look(previous).marked
        && let Some(list) = world.markers.get_mut(&key)
    {
        list.remove(&cell);
    }
}
