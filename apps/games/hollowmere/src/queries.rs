pub mod chat;
pub mod models;
pub mod places;

use crate::data::{
    ABILITIES, BARROW, BIOMES, Biome, BlockKind, CALLINGS, CRACK_STAGES, CRYPT, Cell, DEEP, ELITE,
    FARM, FIELD, FORT, GIVERS, HARBOR, HUNTS, Item, ItemStack, LAKE, Land, MINE, Objective, PLACES,
    QUESTS, Recipe, STARTER, Sheet as CallingSheet, TOWN, ToolKind, WOOD, art_color, cell_index,
};
use crate::resources::{
    Crypt, Generation, Inventory, Mine, Noises, Pad, PlaceSpot, Places, Sheet, Village, VoxelWorld,
    ZonePlan,
};
use crate::theme;
use nalgebra_glm::{IVec3, Vec2, Vec3};
use nightshade_random::prelude::*;
use nightshade_render::data::geometry::{Mesh, Vertex};
use nightshade_sprite::prelude::SpriteRegion;
use nightshade_voxels::prelude::Blocks;
pub use nightshade_voxels::queries::{
    block_index, box_in_liquid, chunk_of, chunks_of, column_levels, column_of, column_settled,
    cube_mesh, facing_of, footing, ground_height, in_bounds, liquid_near, raycast,
    raycast_to_water, slab_of, slide_box, surface_height, walkable, with_bounds,
};
use noise::NoiseFn;

pub fn block_at(world: &VoxelWorld, position: IVec3) -> BlockKind {
    if position.y < theme::FLOOR_Y {
        return BlockKind::Bedrock;
    }
    BlockKind::from_index(nightshade_voxels::queries::block_at(world, position))
}

pub fn selected_stack(inventory: &Inventory) -> Option<ItemStack> {
    inventory.slots[inventory.selected]
}

pub fn inventory_count(inventory: &Inventory, item: Item) -> u32 {
    inventory
        .slots
        .iter()
        .flatten()
        .filter(|stack| stack.item == item)
        .map(|stack| stack.count)
        .sum()
}

pub fn can_craft(inventory: &Inventory, recipe: &Recipe) -> bool {
    recipe
        .inputs
        .iter()
        .all(|(item, count)| inventory_count(inventory, *item) >= *count)
}

pub fn break_seconds(kind: BlockKind, held: Option<Item>) -> f32 {
    let info = kind.info();
    if info.hardness <= 0.0 {
        return 0.0;
    }
    if !info.hardness.is_finite() {
        return f32::INFINITY;
    }
    let base = info.hardness * 1.5;
    match (info.tool, held) {
        (Some(wanted), Some(Item::Tool(tool))) if tool == wanted => base / tool.speed(),
        _ => base,
    }
}

pub fn reserved(generation: &Generation, x: i32, z: i32) -> bool {
    zone_built(generation, x, z)
        || villages_near(generation, x, z).any(|village| {
            village_pads(village).any(|pad| pad_reach(&pad, x, z) == 0)
                || village.roads.contains(&(x, z))
                || village
                    .signs
                    .iter()
                    .any(|(block, _)| block[0] == x && block[2] == z)
        })
}

pub fn near_building(generation: &Generation, x: i32, z: i32, margin: i32) -> bool {
    villages_near(generation, x, z).any(|village| {
        village.buildings.iter().any(|footprint| {
            x >= footprint.x - margin
                && x < footprint.x + footprint.width + margin
                && z >= footprint.z - margin
                && z < footprint.z + footprint.depth + margin
        })
    })
}

pub fn footprint_base(generation: &Generation, x: i32, z: i32) -> Option<i32> {
    villages_near(generation, x, z).find_map(|village| {
        village
            .signs
            .iter()
            .find(|(block, _)| block[0] == x && block[2] == z)
            .map(|(block, _)| block[1] - 1)
            .or_else(|| {
                village_pads(village)
                    .find(|pad| pad_reach(pad, x, z) == 0)
                    .map(|pad| pad.base)
            })
    })
}

pub fn village_pads(village: &Village) -> impl Iterator<Item = Pad> + '_ {
    let half = theme::PLAZA_HALF;
    let plaza = Pad {
        x: village.square.0 - half,
        z: village.square.1 - half,
        width: half * 2 + 1,
        depth: half * 2 + 1,
        base: village.level,
    };
    std::iter::once(plaza).chain(village.buildings.iter().map(|footprint| Pad {
        x: footprint.x,
        z: footprint.z,
        width: footprint.width,
        depth: footprint.depth,
        base: footprint.base,
    }))
}

pub fn pad_reach(pad: &Pad, x: i32, z: i32) -> i32 {
    let across = (pad.x - 2 - x).max(x - (pad.x + pad.width + 1)).max(0);
    let along = (pad.z - 2 - z).max(z - (pad.z + pad.depth + 1)).max(0);
    across.max(along)
}

pub fn apron_share(reach: i32) -> f32 {
    let share = 1.0 - reach as f32 / theme::APRON;
    if share <= 0.0 {
        return 0.0;
    }
    share * share * (3.0 - 2.0 * share)
}

pub fn footprint_apron(generation: &Generation, x: i32, z: i32) -> Option<(i32, f32)> {
    let mut best: Option<(i32, f32)> = None;
    for village in villages_near(generation, x, z) {
        for pad in village_pads(village) {
            let share = apron_share(pad_reach(&pad, x, z));
            if share > 0.0 && best.is_none_or(|(_, held)| share > held) {
                best = Some((pad.base, share));
            }
        }
    }
    best
}

pub fn region_of(x: i32, z: i32) -> (i32, i32) {
    (x.div_euclid(theme::REGION), z.div_euclid(theme::REGION))
}

pub fn villages_near(generation: &Generation, x: i32, z: i32) -> impl Iterator<Item = &Village> {
    let region = region_of(x, z);
    (-1..=1).flat_map(move |dx| {
        (-1..=1).filter_map(move |dz| {
            generation
                .villages
                .get(&(region.0 + dx, region.1 + dz))
                .and_then(|found| found.as_ref())
        })
    })
}

pub fn crack_stage(progress: f32) -> usize {
    ((progress.clamp(0.0, 0.999) * crate::data::CRACK_STAGES.len() as f32) as usize)
        .min(crate::data::CRACK_STAGES.len() - 1)
}

pub fn sheet_tile(sheet: &Sheet, index: u32) -> SpriteRegion {
    sheet
        .tiles
        .cell(theme::SHEET_COLUMNS, theme::SHEET_ROWS, index)
}

pub fn sheet_standing(sheet: &Sheet, index: u32) -> (SpriteRegion, f32) {
    let tile = sheet_tile(sheet, index);
    let empty = sheet
        .empty_rows_below
        .get(index as usize)
        .copied()
        .unwrap_or(0) as f32;
    let share = 1.0 - empty / theme::TILE_TEXELS as f32;
    let region = SpriteRegion::new(
        tile.uv_min,
        Vec2::new(
            tile.uv_max.x,
            tile.uv_min.y + (tile.uv_max.y - tile.uv_min.y) * share,
        ),
    );
    (region, share)
}

pub fn stage_of(progress: f32) -> usize {
    if progress >= 1.0 {
        3
    } else {
        ((progress * 3.0) as usize).min(2)
    }
}

pub fn item_scale(item: Item) -> f32 {
    match item {
        Item::Block(_) => 0.28,
        _ => 0.42,
    }
}

pub fn ripple_pixels() -> Vec<u8> {
    let size = theme::RIPPLE_TEXELS as usize;
    let waves: [(f32, f32, f32); 4] = [
        (1.0, 2.0, 1.0),
        (3.0, 1.0, 0.55),
        (2.0, -3.0, 0.4),
        (-4.0, 2.0, 0.25),
    ];
    let mut pixels = vec![0u8; size * size * 4];
    let step = std::f32::consts::TAU / size as f32;
    for row in 0..size {
        for column in 0..size {
            let mut slope_x = 0.0;
            let mut slope_z = 0.0;
            for (across, along, weight) in waves {
                let phase = (column as f32 * across + row as f32 * along) * step;
                let swing = phase.cos() * weight * theme::RIPPLE_HEIGHT;
                slope_x += swing * across;
                slope_z += swing * along;
            }
            let normal = nalgebra_glm::Vec3::new(-slope_x, 1.0, -slope_z).normalize();
            let offset = (row * size + column) * 4;
            pixels[offset] = ((normal.x * 0.5 + 0.5) * 255.0).round() as u8;
            pixels[offset + 1] = ((normal.z * 0.5 + 0.5) * 255.0).round() as u8;
            pixels[offset + 2] = ((normal.y * 0.5 + 0.5) * 255.0).round() as u8;
            pixels[offset + 3] = 255;
        }
    }
    pixels
}

pub fn extruded_cell_mesh(pixels: &[u8], width: u32, cell: Cell) -> Mesh {
    let texel = 1.0 / 16.0;
    let half_depth = texel * theme::ITEM_DEPTH * 0.5;
    let alpha_at = |column: i32, row: i32| -> u8 {
        if !(0..16).contains(&column) || !(0..16).contains(&row) {
            return 0;
        }
        let x = cell.0 * theme::TILE_TEXELS + column as u32;
        let y = cell.1 * theme::TILE_TEXELS + row as u32;
        pixels
            .get(((y * width + x) * 4 + 3) as usize)
            .copied()
            .unwrap_or(0)
    };
    let mut builder = Builder {
        vertices: Vec::new(),
        indices: Vec::new(),
    };
    let art = Vec2::new(cell_index(cell), 1.0);
    for row in 0..16 {
        for column in 0..16 {
            if alpha_at(column, row) < 128 {
                continue;
            }
            let middle = Vec2::new(
                (column as f32 + 0.5) / theme::TILE_TEXELS as f32,
                (row as f32 + 0.5) / theme::TILE_TEXELS as f32,
            );
            let x0 = column as f32 * texel - 0.5;
            let x1 = x0 + texel;
            let y1 = 0.5 - row as f32 * texel;
            let y0 = y1 - texel;
            let (front, back) = (half_depth, -half_depth);
            let faces = [
                (
                    [
                        Vec3::new(x0, y0, front),
                        Vec3::new(x1, y0, front),
                        Vec3::new(x1, y1, front),
                        Vec3::new(x0, y1, front),
                    ],
                    Vec3::z(),
                    1.0,
                    true,
                ),
                (
                    [
                        Vec3::new(x1, y0, back),
                        Vec3::new(x0, y0, back),
                        Vec3::new(x0, y1, back),
                        Vec3::new(x1, y1, back),
                    ],
                    -Vec3::z(),
                    0.9,
                    true,
                ),
                (
                    [
                        Vec3::new(x0, y0, back),
                        Vec3::new(x0, y0, front),
                        Vec3::new(x0, y1, front),
                        Vec3::new(x0, y1, back),
                    ],
                    -Vec3::x(),
                    0.75,
                    alpha_at(column - 1, row) < 128,
                ),
                (
                    [
                        Vec3::new(x1, y0, front),
                        Vec3::new(x1, y0, back),
                        Vec3::new(x1, y1, back),
                        Vec3::new(x1, y1, front),
                    ],
                    Vec3::x(),
                    0.75,
                    alpha_at(column + 1, row) < 128,
                ),
                (
                    [
                        Vec3::new(x0, y1, front),
                        Vec3::new(x1, y1, front),
                        Vec3::new(x1, y1, back),
                        Vec3::new(x0, y1, back),
                    ],
                    Vec3::y(),
                    1.0,
                    alpha_at(column, row - 1) < 128,
                ),
                (
                    [
                        Vec3::new(x0, y0, back),
                        Vec3::new(x1, y0, back),
                        Vec3::new(x1, y0, front),
                        Vec3::new(x0, y0, front),
                    ],
                    -Vec3::y(),
                    0.7,
                    alpha_at(column, row + 1) < 128,
                ),
            ];
            for (corners, normal, shade, wanted) in faces {
                if !wanted {
                    continue;
                }
                builder.quad(
                    corners,
                    normal,
                    [[middle.x, middle.y]; 4],
                    art,
                    [shade, shade, shade, 1.0],
                );
            }
        }
    }
    builder.finish()
}

pub fn crack_mesh(blocks: &Blocks, stage: usize) -> Mesh {
    let cell = CRACK_STAGES[stage.min(CRACK_STAGES.len() - 1)];
    cube_mesh(
        blocks,
        |_| cell,
        -theme::CRACK_INFLATE,
        1.0 + theme::CRACK_INFLATE,
    )
}

pub fn art_mesh(art: [&'static str; 16]) -> Mesh {
    let texel = 1.0 / 16.0;
    let half_depth = texel * 0.5;
    let filled = |column: i32, row: i32| -> Option<[f32; 4]> {
        if !(0..16).contains(&column) || !(0..16).contains(&row) {
            return None;
        }
        art[row as usize]
            .chars()
            .nth(column as usize)
            .and_then(art_color)
    };
    let mut builder = Builder {
        vertices: Vec::new(),
        indices: Vec::new(),
    };
    let flat = [[0.0, 0.0]; 4];
    for row in 0..16 {
        for column in 0..16 {
            let Some(color) = filled(column, row) else {
                continue;
            };
            let x0 = column as f32 * texel - 0.5;
            let x1 = x0 + texel;
            let y1 = 0.5 - row as f32 * texel;
            let y0 = y1 - texel;
            let (front, back) = (half_depth, -half_depth);
            let side = [color[0] * 0.75, color[1] * 0.75, color[2] * 0.75, 1.0];
            let rear = [color[0] * 0.9, color[1] * 0.9, color[2] * 0.9, 1.0];
            builder.quad(
                [
                    Vec3::new(x0, y0, front),
                    Vec3::new(x1, y0, front),
                    Vec3::new(x1, y1, front),
                    Vec3::new(x0, y1, front),
                ],
                Vec3::z(),
                flat,
                Vec2::new(0.0, 1.0),
                color,
            );
            builder.quad(
                [
                    Vec3::new(x1, y0, back),
                    Vec3::new(x0, y0, back),
                    Vec3::new(x0, y1, back),
                    Vec3::new(x1, y1, back),
                ],
                -Vec3::z(),
                flat,
                Vec2::new(0.0, 1.0),
                rear,
            );
            if filled(column - 1, row).is_none() {
                builder.quad(
                    [
                        Vec3::new(x0, y0, back),
                        Vec3::new(x0, y0, front),
                        Vec3::new(x0, y1, front),
                        Vec3::new(x0, y1, back),
                    ],
                    -Vec3::x(),
                    flat,
                    Vec2::new(0.0, 1.0),
                    side,
                );
            }
            if filled(column + 1, row).is_none() {
                builder.quad(
                    [
                        Vec3::new(x1, y0, front),
                        Vec3::new(x1, y0, back),
                        Vec3::new(x1, y1, back),
                        Vec3::new(x1, y1, front),
                    ],
                    Vec3::x(),
                    flat,
                    Vec2::new(0.0, 1.0),
                    side,
                );
            }
            if filled(column, row - 1).is_none() {
                builder.quad(
                    [
                        Vec3::new(x0, y1, front),
                        Vec3::new(x1, y1, front),
                        Vec3::new(x1, y1, back),
                        Vec3::new(x0, y1, back),
                    ],
                    Vec3::y(),
                    flat,
                    Vec2::new(0.0, 1.0),
                    side,
                );
            }
            if filled(column, row + 1).is_none() {
                builder.quad(
                    [
                        Vec3::new(x0, y0, back),
                        Vec3::new(x1, y0, back),
                        Vec3::new(x1, y0, front),
                        Vec3::new(x0, y0, front),
                    ],
                    -Vec3::y(),
                    flat,
                    Vec2::new(0.0, 1.0),
                    side,
                );
            }
        }
    }
    builder.finish()
}

pub fn hash(seed: u32, x: i32, z: i32) -> u32 {
    let mut value = seed
        ^ (x as u32).wrapping_mul(0x9E37_79B1)
        ^ (z as u32).wrapping_mul(0x85EB_CA77).rotate_left(16);
    value ^= value >> 15;
    value = value.wrapping_mul(0x2C1B_3C6D);
    value ^= value >> 12;
    value = value.wrapping_mul(0x297A_2D39);
    value ^= value >> 15;
    value.max(1)
}

pub fn biome_weights(noises: &Noises, x: i32, z: i32) -> [f32; BIOMES] {
    let point = [x as f64, z as f64];
    let warm = noises.temperature.get(point);
    let wet = noises.moisture.get(point);
    let high = noises.hills.get(point);
    let deep = ramp(-noises.sea.get(point), 0.02, 0.34);
    let dry = 1.0 - deep;
    let hot = ramp(warm, 0.15, 0.5);
    let cold = ramp(-warm, 0.12, 0.42);
    let mild = (1.0 - hot.max(cold)).max(0.0);
    let damp = ramp(wet, 0.05, 0.4);
    let arid = ramp(-wet, 0.0, 0.35);
    let hill = ramp(high, 0.2, 0.55);
    let crest = ramp(high, 0.52, 0.85);
    let hollow = ramp(-high, 0.05, 0.4);
    let desert = hot * arid * (1.0 - crest);
    let badlands = hot * arid * hill * 1.4;
    let swamp = damp * hollow * (1.0 - cold) * 1.5;
    let snowfield = cold * (1.0 - crest) * 1.2;
    let peaks = crest * (1.0 + cold);
    let forest = damp * mild * (1.0 - desert);
    let hills = hill * mild * (1.0 - crest);
    let plains = (mild - desert.max(forest).max(hills).max(swamp)).max(0.08);
    let mut weights = [
        plains * dry,
        forest * dry,
        desert * dry,
        hills * dry,
        deep * 2.2,
        snowfield * dry,
        peaks * dry,
        swamp * dry,
        badlands * dry,
    ];
    let total: f32 = weights.iter().sum();
    for weight in &mut weights {
        *weight /= total.max(0.0001);
    }
    weights
}

pub fn dominant_biome(weights: [f32; BIOMES]) -> Biome {
    let mut best = 0;
    for index in 1..BIOMES {
        if weights[index] > weights[best] {
            best = index;
        }
    }
    Biome::ALL[best]
}

pub fn biome_at(generation: &Generation, x: i32, z: i32) -> Biome {
    dominant_biome(biome_weights(&generation.noises, x, z))
}

pub fn dungeon_at(seed: u32, region: (i32, i32)) -> Option<(IVec3, i32)> {
    let mut rng = Random::seeded((hash(seed ^ 0x0DEA_D00D, region.0, region.1)) as u64);
    if rng.u32(..) % 100 >= theme::DUNGEON_CHANCE {
        return None;
    }
    let span = theme::DUNGEON_ROOMS * theme::DUNGEON_ROOM;
    let corner = IVec3::new(
        region.0 * theme::REGION + whole(&mut rng, 20, theme::REGION - span - 20),
        whole(&mut rng, theme::DUNGEON_LOW, theme::DUNGEON_HIGH),
        region.1 * theme::REGION + whole(&mut rng, 20, theme::REGION - span - 20),
    );
    Some((corner, span))
}

pub fn dungeon_chest(seed: u32, region: (i32, i32)) -> Option<IVec3> {
    let (corner, span) = dungeon_at(seed, region)?;
    Some(IVec3::new(
        corner.x + span / 2,
        corner.y + 1,
        corner.z + span / 2,
    ))
}

pub fn dungeon_loot(seed: u32, block: IVec3) -> Option<Vec<Option<ItemStack>>> {
    if dungeon_chest(seed, region_of(block.x, block.z)) != Some(block) {
        return None;
    }
    let prizes = [
        (Item::Block(BlockKind::DiamondOre), 1, 3),
        (Item::Block(BlockKind::GoldOre), 2, 6),
        (Item::Block(BlockKind::IronOre), 3, 8),
        (Item::Block(BlockKind::Glowstone), 2, 6),
        (Item::Block(BlockKind::Tnt), 1, 4),
        (Item::Block(BlockKind::Obsidian), 1, 3),
        (Item::Tool(ToolKind::Sword), 1, 1),
        (Item::Block(BlockKind::Bricks), 4, 10),
    ];
    let mut rng = Random::seeded((hash(seed ^ 0x10AD_5EED, block.x, block.z)) as u64);
    let mut slots = vec![None; theme::CHEST_SLOTS];
    for _ in 0..theme::DUNGEON_PRIZES {
        let prize = prizes[rng.u32(..) as usize % prizes.len()];
        let count = whole(&mut rng, prize.1, prize.2 + 1) as u32;
        let slot = rng.u32(..) as usize % theme::CHEST_SLOTS;
        slots[slot] = Some(ItemStack {
            item: prize.0,
            count,
        });
    }
    Some(slots)
}

struct Builder {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
}

impl Builder {
    fn quad(
        &mut self,
        corners: [Vec3; 4],
        normal: Vec3,
        uvs: [[f32; 2]; 4],
        art: Vec2,
        color: [f32; 4],
    ) {
        let base = self.vertices.len() as u32;
        for (index, corner) in corners.iter().enumerate() {
            self.vertices.push(Vertex::with_all(
                *corner,
                normal,
                uvs[index],
                [art.x, art.y],
                [1.0, 0.0, 0.0, 1.0],
                color,
            ));
        }
        self.indices
            .extend([0, 1, 2, 0, 2, 3].iter().map(|offset| base + offset));
    }

    fn finish(self) -> Mesh {
        with_bounds(Mesh::new(self.vertices, self.indices))
    }
}

pub fn ramp(value: f64, start: f64, end: f64) -> f32 {
    ((value - start) / (end - start)).clamp(0.0, 1.0) as f32
}

pub fn biome_and_rock(noises: &Noises, x: i32, z: i32) -> (Biome, f32) {
    let weights = biome_weights(noises, x, z);
    let mut amount = 0.0;
    for (biome, weight) in Biome::ALL.iter().zip(weights) {
        amount += weight * biome.rockiness();
    }
    (dominant_biome(weights), amount)
}

pub fn footprint_flatten(generation: &Generation, x: i32, z: i32) -> f32 {
    if generation.plan.land == Land::Flat || reserved(generation, x, z) {
        return 1.0;
    }
    let mut most = zone_flatten(&generation.zones, x, z);
    if let Some((_, share)) = footprint_apron(generation, x, z) {
        most = most.max(share);
    }
    most
}

pub fn ground_top(generation: &Generation, x: i32, z: i32) -> Option<i32> {
    let heights = generation.heights.as_ref()?;
    let offset_x = x - heights.column[0] * theme::CHUNK;
    let offset_z = z - heights.column[1] * theme::CHUNK;
    if !(0..theme::CHUNK).contains(&offset_x) || !(0..theme::CHUNK).contains(&offset_z) {
        return None;
    }
    let top = *heights
        .tops
        .get((offset_z * theme::CHUNK + offset_x) as usize)?;
    (top >= theme::FLOOR_Y).then_some(top)
}

pub fn shelf_height(noises: &Noises, x: i32, z: i32, height: i32, rock: f32) -> f32 {
    let bite =
        (ramp(noises.shelf.get([x as f64, z as f64]), -0.15, 0.45) * rock * theme::SHELF_BITE)
            .clamp(0.0, 1.0);
    let stepped = (height as f32 / theme::SHELF_STEP).floor() * theme::SHELF_STEP;
    height as f32 * (1.0 - bite) + stepped * bite
}

pub fn rock_density(
    noises: &Noises,
    x: i32,
    y: i32,
    z: i32,
    surface: f32,
    rock: f32,
    lift: f32,
) -> f32 {
    let relief = theme::ROCK_RELIEF * (theme::ROCK_FLOOR + rock);
    let above = theme::SHAPE_UP * relief * lift;
    let below = theme::SHAPE_DOWN * relief;
    let base = surface - y as f32;
    if base > below || base < -above {
        return base;
    }
    let ridge = 1.0 - 2.0 * noises.stone.get([x as f64, y as f64, z as f64]).abs() as f32;
    let crag = noises.crag.get([
        x as f64 / theme::CRAG_SCALE,
        y as f64 / (theme::CRAG_SCALE * 0.7),
        z as f64 / theme::CRAG_SCALE,
    ]) as f32;
    let shape = (ridge - 0.5) * theme::ROCK_RIDGE + crag * theme::ROCK_CRAG;
    let reach = if base < 0.0 {
        let share = base / above;
        lift * (1.0 - share * share)
    } else {
        1.0
    };
    base + shape * relief * reach
}

pub fn chip_at(noises: &Noises, x: i32, y: i32, z: i32) -> f32 {
    noises.chip.get([x as f64, y as f64, z as f64]) as f32
}

pub fn cave_field(noises: &Noises, x: i32, y: i32, z: i32) -> [f32; 3] {
    let point = [x as f64 / 13.0, y as f64 / 9.0, z as f64 / 13.0];
    [
        noises.worm.get(point) as f32,
        noises
            .twist
            .get([point[0] + 31.0, point[1], point[2] + 17.0]) as f32,
        noises.hollow.get([x as f64, y as f64 * 1.6, z as f64]) as f32,
    ]
}

pub fn ground_at(
    world: &VoxelWorld,
    x: i32,
    z: i32,
    from: f32,
    rise: i32,
    fall: i32,
) -> Option<f32> {
    footing(world, x, z, from, rise, fall).map(|floor| floor as f32 + 1.0)
}

pub fn stand_at(world: &VoxelWorld, spot: Vec2, from: f32) -> Option<f32> {
    let (x, z) = (spot.x.floor() as i32, spot.y.floor() as i32);
    ground_at(
        world,
        x,
        z,
        from,
        theme::NPC_SETTLE_RISE,
        theme::NPC_SETTLE_FALL,
    )
    .or_else(|| walkable(world, x, z).map(|floor| floor as f32 + 1.0))
}

pub fn guess_height(generation: &Generation, x: i32, z: i32) -> i32 {
    footprint_base(generation, x, z).unwrap_or_else(|| land_height(generation, x, z) - 1)
}

pub fn raw_height(generation: &Generation, x: i32, z: i32) -> i32 {
    let noises = &generation.noises;
    let plan = &generation.plan;
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
    let river = noises.river.get(point).abs() as f32;
    if river < theme::RIVER_EDGE && plan.land != Land::Flat {
        let above = height - theme::SEA_LEVEL as f32;
        let tall = smooth_step(theme::RIVER_LOW, theme::RIVER_HIGH, above);
        let bed = (theme::SEA_LEVEL - theme::RIVER_BED) as f32
            - 2.0 * (1.0 - river / theme::RIVER_WIDTH).max(0.0);
        let weight = smooth_step(theme::RIVER_WIDTH, theme::RIVER_EDGE, river).max(tall);
        let carved = height.min(bed);
        height = carved + (height - carved) * weight;
    }
    (height.round() as i32).clamp(theme::FLOOR_Y + 6, theme::WORLD_Y - plan.land.headroom())
}

pub fn guessed_ground(generation: &Generation, spot: Vec2) -> Vec3 {
    if let Some(floor) = zone_floor(generation, spot) {
        return Vec3::new(spot.x, floor as f32, spot.y);
    }
    let (x, z) = (spot.x.floor() as i32, spot.y.floor() as i32);
    let height = guess_height(generation, x, z).max(theme::SEA_LEVEL);
    Vec3::new(spot.x, height as f32 + 1.0, spot.y)
}

pub fn under_water(generation: &Generation, spot: Vec2) -> bool {
    guess_height(generation, spot.x.floor() as i32, spot.y.floor() as i32) < theme::SEA_LEVEL
}

pub fn flat(position: Vec3) -> Vec2 {
    Vec2::new(position.x, position.z)
}

pub fn flat_distance(first: Vec3, second: Vec3) -> f32 {
    (flat(first) - flat(second)).magnitude()
}

pub fn heading_toward(from: Vec3, to: Vec3) -> f32 {
    (from.x - to.x).atan2(from.z - to.z)
}

pub fn is_alive(health: f32) -> bool {
    health > 0.0
}

pub fn level_threshold(level: u32) -> u32 {
    80 + 45 * level
}

pub fn sim_max_health(level: u32) -> f32 {
    90.0 + 10.0 * level as f32
}

pub fn sim_strength(level: u32) -> f32 {
    9.0 + 1.4 * level as f32
}

pub fn hunt_for_level(level: u32) -> usize {
    HUNTS
        .iter()
        .rposition(|hunt| level >= hunt.min_level)
        .unwrap_or(0)
}

pub fn person_plate(name: &str, guild: &str, level: u32) -> String {
    format!("<{guild}> {level}\n{name}")
}

pub fn player_plate(name: &str, level: u32) -> String {
    format!("Level {level}\n{name}")
}

pub fn mob_plate(name: &str, level: u32, rank: u8) -> String {
    let tag = if rank >= ELITE { " Elite" } else { "" };
    format!("Level {level}{tag}\n{name}")
}

pub fn giver_plate(giver: usize, mark: u8) -> String {
    let data = &GIVERS[giver];
    let sign = match mark {
        1 => "!",
        2 => "?",
        _ => "",
    };
    format!("<{}>\n{}\n{sign}", data.title, data.name)
}

pub fn place_spot(places: &Places, place: usize) -> PlaceSpot {
    places.spots.get(place).copied().unwrap_or(PlaceSpot {
        center: Vec2::new(places.town.0 as f32, places.town.1 as f32),
        radius: 20.0,
        floor: None,
    })
}

pub fn spot_of(places: &Places, place: usize, offset: Vec2) -> Vec2 {
    place_spot(places, place).center + offset
}

pub fn place_at(places: &Places, position: Vec3) -> Option<usize> {
    let spot = flat(position);
    let mut best = None;
    let mut best_fraction = 1.0;
    for (index, place) in places.spots.iter().enumerate() {
        let fraction = (spot - place.center).magnitude() / place.radius;
        let Some(floor) = place.floor else {
            continue;
        };
        if fraction < 1.0 && (position.y - floor).abs() < 5.0 {
            return Some(index);
        }
    }
    for (index, place) in places.spots.iter().enumerate() {
        if place.floor.is_some() {
            continue;
        }
        let fraction = (spot - place.center).magnitude() / place.radius;
        if fraction < best_fraction {
            best_fraction = fraction;
            best = Some(index);
        }
    }
    best
}

pub fn hour_text(hour: f32) -> String {
    let total_minutes = ((hour.rem_euclid(24.0)) * 60.0) as u32;
    format!("{:02}:{:02}", total_minutes / 60, total_minutes % 60)
}

pub fn coin_text(copper: u32) -> String {
    let gold = copper / 10_000;
    let silver = (copper / 100) % 100;
    let copper = copper % 100;
    if gold > 0 {
        format!("{gold}g {silver}s {copper}c")
    } else if silver > 0 {
        format!("{silver}s {copper}c")
    } else {
        format!("{copper}c")
    }
}

pub fn night_amount(hour: f32) -> f32 {
    let hour = hour.rem_euclid(24.0);
    if hour < 5.0 || hour > 21.0 {
        1.0
    } else if hour < 7.0 {
        1.0 - (hour - 5.0) / 2.0
    } else if hour > 19.0 {
        (hour - 19.0) / 2.0
    } else {
        0.0
    }
}

pub fn offline_fraction(hour: f32) -> f32 {
    let hour = hour.rem_euclid(24.0);
    if !(6.0..=23.0).contains(&hour) {
        theme::OFFLINE_FRACTION_NIGHT
    } else if hour > 20.5 || hour < 8.0 {
        theme::OFFLINE_FRACTION_EVENING
    } else {
        0.0
    }
}

pub fn calling_sheet(class: usize) -> CallingSheet {
    let data = &CALLINGS[class];
    CallingSheet {
        title: data.name.to_string(),
        blurb: data.blurb.to_string(),
        stats: [
            format!("Health {}", data.health),
            format!("Mana {}", data.mana),
            format!("Strength {}", data.strength),
            format!("Speed {:.1}", data.speed),
        ],
        abilities: data.abilities.map(|pick| ABILITIES[pick].name.to_string()),
    }
}

pub fn hero_box(position: Vec3) -> (Vec3, Vec3) {
    (
        Vec3::new(
            position.x - theme::PLAYER_HALF_WIDTH,
            position.y,
            position.z - theme::PLAYER_HALF_WIDTH,
        ),
        Vec3::new(
            position.x + theme::PLAYER_HALF_WIDTH,
            position.y + theme::PLAYER_HEIGHT,
            position.z + theme::PLAYER_HALF_WIDTH,
        ),
    )
}

pub fn land_height(generation: &Generation, x: i32, z: i32) -> i32 {
    land_height_with(generation, &generation.zones, x, z)
}

pub fn land_height_with(generation: &Generation, zones: &[ZonePlan], x: i32, z: i32) -> i32 {
    let raw = raw_height(generation, x, z);
    if zones.is_empty() {
        return raw;
    }
    let shaped = zone_relief(zones, x, z, raw as f32).round() as i32;
    shaped.clamp(
        theme::FLOOR_Y + 6,
        theme::WORLD_Y - generation.plan.land.headroom(),
    )
}

pub fn zone_center(zone: &ZonePlan) -> Vec2 {
    Vec2::new(zone.center.0 as f32 + 0.5, zone.center.1 as f32 + 0.5)
}

pub fn direction(angle: f32) -> Vec2 {
    Vec2::new(angle.cos(), angle.sin())
}

pub fn axis_of(angle: f32) -> (i32, i32) {
    let (sine, cosine) = angle.sin_cos();
    if cosine.abs() >= sine.abs() {
        (cosine.signum() as i32, 0)
    } else {
        (0, sine.signum() as i32)
    }
}

pub fn smooth_step(low: f32, high: f32, value: f32) -> f32 {
    let share = ((value - low) / (high - low)).clamp(0.0, 1.0);
    share * share * (3.0 - 2.0 * share)
}

pub fn along_segment(point: Vec2, start: Vec2, end: Vec2) -> (f32, f32) {
    let run = end - start;
    let length = run.magnitude();
    if length < 1e-4 {
        return (0.0, (point - start).magnitude());
    }
    let unit = run / length;
    let offset = point - start;
    let along = offset.dot(&unit);
    let clamped = along.clamp(0.0, length);
    let nearest = start + unit * clamped;
    (along, (point - nearest).magnitude())
}

fn flat_band(index: usize, zone: &ZonePlan) -> Option<(f32, f32)> {
    if index == TOWN {
        return Some((
            zone.radius + theme::TOWN_LIP,
            zone.radius + theme::TOWN_SLOPE,
        ));
    }
    let radius = match index {
        FIELD => zone.radius,
        FORT => theme::FORT_PAD,
        FARM => theme::FARM_PAD,
        HARBOR => theme::HARBOR_PAD,
        STARTER => theme::HAMLET_PAD,
        WOOD => theme::CLEARING,
        _ => return None,
    };
    Some((
        radius * theme::ZONE_FLAT_INNER,
        radius * theme::ZONE_FLAT_OUTER,
    ))
}

pub fn zone_relief(zones: &[ZonePlan], x: i32, z: i32, mut height: f32) -> f32 {
    let point = Vec2::new(x as f32 + 0.5, z as f32 + 0.5);
    for (index, zone) in zones.iter().enumerate() {
        let offset = point - zone_center(zone);
        let distance = offset.magnitude();
        let share = distance / zone.radius;
        match index {
            LAKE => {
                if share < 1.3 {
                    let bowl = (1.0 - share * share).max(0.0);
                    let want = theme::SEA_LEVEL as f32 - 1.0 - theme::LAKE_DEPTH * bowl;
                    let lowered = height.min(want);
                    let weight = smooth_step(0.8, 1.2, share);
                    height = lowered + (height - lowered) * weight;
                }
            }
            MINE => {
                if share < 1.0 {
                    let bump = 1.0 - share * share;
                    height += theme::MINE_HILL * bump * bump.sqrt();
                }
            }
            BARROW => {
                for (mound_x, mound_z) in theme::MOUNDS {
                    let reach =
                        (offset - Vec2::new(mound_x, mound_z)).magnitude() / theme::MOUND_RADIUS;
                    if reach < 1.0 {
                        height += theme::MOUND_HEIGHT * (1.0 - reach * reach);
                    }
                }
            }
            _ => {
                if let Some((inner, outer)) = flat_band(index, zone)
                    && distance < outer
                {
                    let weight = smooth_step(inner, outer, distance);
                    let level = zone.level as f32;
                    height = level + (height - level) * weight;
                }
            }
        }
    }
    height
}

pub fn zone_flatten(zones: &[ZonePlan], x: i32, z: i32) -> f32 {
    let point = Vec2::new(x as f32 + 0.5, z as f32 + 0.5);
    let mut most = 0.0f32;
    for (index, zone) in zones.iter().enumerate() {
        let Some((inner, outer)) = flat_band(index, zone) else {
            continue;
        };
        let distance = (point - zone_center(zone)).magnitude();
        most = most.max(1.0 - smooth_step(inner, outer, distance));
    }
    most
}

pub fn zone(generation: &Generation, index: usize) -> Option<ZonePlan> {
    generation.zones.get(index).copied()
}

pub fn crypt_layout(generation: &Generation) -> Option<Crypt> {
    let crypt = zone(generation, CRYPT)?;
    let span = theme::DUNGEON_ROOMS * theme::DUNGEON_ROOM;
    let corner = IVec3::new(
        crypt.center.0 - span / 2,
        crypt.level,
        crypt.center.1 - span / 2,
    );
    let (axis_x, axis_z) = axis_of(crypt.facing);
    let axis = Vec2::new(axis_x as f32, axis_z as f32);
    let stair_bottom = zone_center(&crypt) + axis * (span / 2 + theme::DUNGEON_TUNNEL) as f32;
    let stair_top = stair_bottom + axis * theme::STAIR_RUN as f32;
    let top = land_height(
        generation,
        stair_top.x.floor() as i32,
        stair_top.y.floor() as i32,
    );
    Some(Crypt {
        corner,
        span,
        stair_bottom,
        stair_top,
        top,
    })
}

pub fn mine_layout(generation: &Generation) -> Option<Mine> {
    let mine = zone(generation, MINE)?;
    let deep = zone(generation, DEEP)?;
    let mouth = zone_center(&mine) + direction(mine.facing) * (mine.radius * 0.55);
    let mouth_y = land_height(generation, mouth.x.floor() as i32, mouth.y.floor() as i32);
    Some(Mine {
        mouth,
        end: zone_center(&deep),
        mouth_y,
        end_y: deep.level,
    })
}

pub fn harbor_pier(generation: &Generation) -> Option<(Vec2, Vec2)> {
    let harbor = zone(generation, HARBOR)?;
    let lake = zone(generation, LAKE)?;
    let toward = zone_center(&lake) - zone_center(&harbor);
    let length = toward.magnitude();
    if length < 1e-3 {
        return None;
    }
    let unit = toward / length;
    let start = zone_center(&harbor) + unit * 2.0;
    Some((start, start + unit * theme::PIER_LENGTH))
}

pub const ROAD_ZONES: [usize; 8] = [FIELD, WOOD, FORT, BARROW, FARM, MINE, STARTER, HARBOR];

pub fn road_gate(generation: &Generation, target: Vec2) -> Option<Vec2> {
    let town = zone(generation, TOWN)?;
    let center = zone_center(&town);
    let toward = target - center;
    let (axis_x, axis_z) = axis_of(toward.y.atan2(toward.x));
    Some(center + Vec2::new(axis_x as f32, axis_z as f32) * (town.radius + 1.0))
}

pub fn road_near(generation: &Generation, point: Vec2, reach: f32) -> bool {
    road_at(generation, point).is_some_and(|(_, perpendicular)| perpendicular <= reach)
}

fn nearer(best: &mut Option<(f32, f32)>, found: (f32, f32)) {
    if best.is_none_or(|held| found.1 < held.1) {
        *best = Some(found);
    }
}

pub fn road_at(generation: &Generation, point: Vec2) -> Option<(f32, f32)> {
    let mut best: Option<(f32, f32)> = None;
    if let Some(town) = zone(generation, TOWN) {
        let center = zone_center(&town);
        for (axis_x, axis_z) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let gate = center + Vec2::new(axis_x as f32, axis_z as f32) * (town.radius + 1.0);
            nearer(&mut best, along_segment(point, center, gate));
        }
        for index in ROAD_ZONES {
            let Some(target) = zone(generation, index) else {
                continue;
            };
            let target = zone_center(&target);
            let Some(gate) = road_gate(generation, target) else {
                continue;
            };
            nearer(&mut best, along_segment(point, gate, target));
        }
    }
    for (from, to) in village_links(generation, point.x.floor() as i32, point.y.floor() as i32) {
        nearer(&mut best, along_segment(point, from, to));
    }
    best
}

pub fn street_end(village: &Village, toward: Vec2) -> Vec2 {
    let center = Vec2::new(village.square.0 as f32 + 0.5, village.square.1 as f32 + 0.5);
    let offset = toward - center;
    let (axis_x, axis_z) = axis_of(offset.y.atan2(offset.x));
    let arm = match (axis_x, axis_z) {
        (1, 0) => 0,
        (-1, 0) => 1,
        (0, 1) => 2,
        _ => 3,
    };
    let length = village.streets[arm].max(theme::PLAZA_HALF + 1) as f32;
    center + Vec2::new(axis_x as f32, axis_z as f32) * length
}

pub fn village_links(generation: &Generation, x: i32, z: i32) -> Vec<(Vec2, Vec2)> {
    let villages: Vec<&Village> = villages_near(generation, x, z).collect();
    let centers: Vec<Vec2> = villages
        .iter()
        .map(|village| Vec2::new(village.square.0 as f32 + 0.5, village.square.1 as f32 + 0.5))
        .collect();
    let mut links: Vec<(Vec2, Vec2)> = Vec::new();
    for (index, from) in centers.iter().enumerate() {
        let mut others: Vec<(f32, usize)> = centers
            .iter()
            .enumerate()
            .filter(|(other, _)| *other != index)
            .map(|(other, spot)| ((spot - from).magnitude(), other))
            .collect();
        others.sort_by(|left, right| left.0.total_cmp(&right.0));
        for (rank, (distance, other)) in others.iter().enumerate() {
            if rank > 1 || (rank == 1 && *distance > theme::LINK_REACH) {
                break;
            }
            let start = street_end(villages[index], centers[*other]);
            let end = street_end(villages[*other], *from);
            let (low, high) = if (start.x, start.y) <= (end.x, end.y) {
                (start, end)
            } else {
                (end, start)
            };
            if !links.iter().any(|(a, b)| *a == low && *b == high) {
                links.push((low, high));
            }
        }
    }
    links
}

pub fn zone_built(generation: &Generation, x: i32, z: i32) -> bool {
    let point = Vec2::new(x as f32 + 0.5, z as f32 + 0.5);
    if road_near(generation, point, theme::ROAD_HALF + 1.0) {
        return true;
    }
    for (index, zone) in generation.zones.iter().enumerate() {
        let offset = point - zone_center(zone);
        let distance = offset.magnitude();
        if distance > zone.radius + 44.0 {
            continue;
        }
        let built = match index {
            TOWN => (distance - zone.radius).abs() <= 2.5,
            FORT => {
                offset.x.abs() <= theme::FORT_HALF as f32 + 1.5
                    && offset.y.abs() <= theme::FORT_HALF as f32 + 1.5
            }
            FARM => offset.x.abs() <= 13.5 && offset.y.abs() <= 16.5,
            HARBOR => distance <= zone.radius,
            STARTER => distance <= theme::HAMLET_CLEARING,
            WOOD => distance <= theme::CLEARING,
            MINE => {
                mine_layout(generation).is_some_and(|mine| (point - mine.mouth).magnitude() <= 5.0)
            }
            CRYPT => crypt_layout(generation).is_some_and(|crypt| {
                along_segment(point, crypt.stair_bottom, crypt.stair_top).1 <= 2.5
            }),
            _ => false,
        };
        if built {
            return true;
        }
    }
    false
}

pub fn tree_share(generation: &Generation, x: i32, z: i32) -> f32 {
    let point = Vec2::new(x as f32 + 0.5, z as f32 + 0.5);
    let mut share = 1.0;
    for (index, zone) in generation.zones.iter().enumerate() {
        let distance = (point - zone_center(zone)).magnitude();
        if distance > zone.radius {
            continue;
        }
        share = match index {
            WOOD | STARTER => theme::WOOD_TREES,
            FIELD | LAKE => 0.0,
            TOWN => theme::TOWN_TREES,
            _ => share,
        };
    }
    share
}

pub fn zone_floor(generation: &Generation, spot: Vec2) -> Option<i32> {
    for (index, reach) in [(CRYPT, None), (DEEP, Some(theme::DEEP_RADIUS))] {
        let Some(zone) = zone(generation, index) else {
            continue;
        };
        let radius = reach.unwrap_or(zone.radius);
        if (spot - zone_center(&zone)).magnitude() <= radius {
            return Some(zone.level + 1);
        }
    }
    None
}

pub fn zone_signs(generation: &Generation) -> Vec<((i32, i32), String)> {
    let mut signs = Vec::new();
    for (index, zone) in generation.zones.iter().enumerate() {
        if matches!(index, TOWN | LAKE | DEEP | CRYPT) {
            continue;
        }
        let toward = direction(zone.facing);
        let across = Vec2::new(-toward.y, toward.x);
        let spot = zone_center(zone) + toward * (zone.radius * 0.9) + across * 2.5;
        signs.push((
            (spot.x.floor() as i32, spot.y.floor() as i32),
            PLACES[index].name.to_string(),
        ));
    }
    if let Some(crypt) = generation.crypt {
        let toward = (crypt.stair_top - crypt.stair_bottom).normalize();
        let across = Vec2::new(-toward.y, toward.x);
        let spot = crypt.stair_top + toward * 1.5 + across * 2.5;
        signs.push((
            (spot.x.floor() as i32, spot.y.floor() as i32),
            PLACES[CRYPT].name.to_string(),
        ));
    }
    signs
}

pub fn zone_chests(generation: &Generation) -> Vec<IVec3> {
    let mut chests = Vec::new();
    if let Some(crypt) = generation.crypt {
        chests.push(IVec3::new(
            crypt.corner.x + crypt.span / 2,
            crypt.corner.y + 1,
            crypt.corner.z + crypt.span / 2,
        ));
    }
    if let Some(deep) = zone(generation, DEEP) {
        chests.push(IVec3::new(deep.center.0 + 3, deep.level + 1, deep.center.1));
    }
    if let Some(barrow) = zone(generation, BARROW) {
        for (mound_x, mound_z) in theme::MOUNDS {
            let x = barrow.center.0 + mound_x as i32;
            let z = barrow.center.1 + mound_z as i32;
            chests.push(IVec3::new(x, raw_height(generation, x, z) + 1, z));
        }
    }
    if let Some(fort) = zone(generation, FORT) {
        chests.push(IVec3::new(fort.center.0, fort.level + 1, fort.center.1 + 1));
    }
    chests
}

pub fn chest_loot(generation: &Generation, block: IVec3) -> Vec<Option<ItemStack>> {
    if let Some(loot) = dungeon_loot(generation.seed, block) {
        return loot;
    }
    if !zone_chests(generation).contains(&block) {
        return vec![None; theme::CHEST_SLOTS];
    }
    let prizes = [
        (Item::Loot(0), 2, 5),
        (Item::Loot(1), 2, 5),
        (Item::Loot(2), 1, 3),
        (Item::Block(BlockKind::GoldOre), 2, 5),
        (Item::Block(BlockKind::IronOre), 3, 7),
        (Item::Block(BlockKind::Glowstone), 2, 5),
        (Item::Tool(ToolKind::Sword), 1, 1),
        (Item::Block(BlockKind::DiamondOre), 1, 2),
    ];
    let mut rng = Random::seeded((hash(generation.seed ^ 0x2C4E_57F1, block.x, block.z)) as u64);
    let mut slots = vec![None; theme::CHEST_SLOTS];
    for _ in 0..theme::DUNGEON_PRIZES + 1 {
        let prize = prizes[rng.u32(..) as usize % prizes.len()];
        let count = whole(&mut rng, prize.1, prize.2 + 1) as u32;
        let slot = rng.u32(..) as usize % theme::CHEST_SLOTS;
        slots[slot] = Some(ItemStack {
            item: prize.0,
            count,
        });
    }
    slots
}

pub fn turn_in_giver(quest: usize) -> usize {
    match QUESTS[quest].objective {
        Objective::Deliver { giver } => giver,
        _ => QUESTS[quest].giver,
    }
}

pub fn in_the_rooms(inside: (i32, i32), span: i32) -> bool {
    (0..span).contains(&inside.0) && (0..span).contains(&inside.1)
}

pub fn in_the_tunnel(inside: (i32, i32), span: i32) -> bool {
    let along_x = (0..span).contains(&inside.1) && door_band(inside.1, span);
    let along_z = (0..span).contains(&inside.0) && door_band(inside.0, span);
    along_x || along_z
}

fn door_band(value: i32, span: i32) -> bool {
    (value - span / 2).abs() <= 1
}

pub fn room_wall(inside: (i32, i32), span: i32) -> bool {
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
