use crate::data::{
    BIOMES, Biome, BlockKind, CRACK_STAGES, Cell, Item, ItemStack, Land, Recipe, ToolKind,
    art_color, cell_index,
};
use crate::resources::{Generation, Inventory, Noises, Sheet, Village, VoxelWorld};
use crate::theme;
use nalgebra_glm::{IVec3, Vec2, Vec3};
use nightshade_random::prelude::*;
use nightshade_render::data::geometry::{Mesh, Vertex};
use nightshade_sprite::prelude::SpriteRegion;
use nightshade_voxels::prelude::Blocks;
pub use nightshade_voxels::queries::{
    block_index, box_in_liquid, chunk_of, chunks_of, column_levels, column_of, column_settled,
    cube_mesh, facing_of, footing, ground_height, in_bounds, is_solid_at, liquid_near,
    liquid_surface_near, raycast, raycast_to_water, slab_of, slide_box, surface_height, walkable,
    with_bounds,
};
use noise::NoiseFn;

pub fn boat_mesh() -> Mesh {
    let plank = Vec3::new(0.42, 0.28, 0.16);
    let dark = Vec3::new(0.28, 0.18, 0.10);
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let hull = [
        (Vec3::new(-0.7, 0.0, -1.5), Vec3::new(1.4, 0.18, 3.0), plank),
        (Vec3::new(-0.7, 0.0, -1.5), Vec3::new(0.16, 0.55, 3.0), dark),
        (Vec3::new(0.54, 0.0, -1.5), Vec3::new(0.16, 0.55, 3.0), dark),
        (Vec3::new(-0.7, 0.0, -1.5), Vec3::new(1.4, 0.55, 0.16), dark),
        (Vec3::new(-0.7, 0.0, 1.34), Vec3::new(1.4, 0.55, 0.16), dark),
        (Vec3::new(-0.7, 0.3, -0.2), Vec3::new(1.4, 0.14, 0.5), plank),
    ];
    for (corner, span, color) in hull {
        add_box(&mut vertices, &mut indices, corner, span, color);
    }
    Mesh::new(vertices, indices)
}

fn add_box(
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u32>,
    corner: Vec3,
    span: Vec3,
    color: Vec3,
) {
    let across = Vec3::new(span.x, 0.0, 0.0);
    let up = Vec3::new(0.0, span.y, 0.0);
    let along = Vec3::new(0.0, 0.0, span.z);
    let faces = [
        (corner + up, along, across, Vec3::new(0.0, 1.0, 0.0)),
        (corner, across, along, Vec3::new(0.0, -1.0, 0.0)),
        (corner + across, up, along, Vec3::new(1.0, 0.0, 0.0)),
        (corner, along, up, Vec3::new(-1.0, 0.0, 0.0)),
        (corner + along, across, up, Vec3::new(0.0, 0.0, 1.0)),
        (corner, up, across, Vec3::new(0.0, 0.0, -1.0)),
    ];
    for (origin, first, second, normal) in faces {
        let base = vertices.len() as u32;
        for point in [
            origin,
            origin + first,
            origin + first + second,
            origin + second,
        ] {
            vertices.push(Vertex::with_all(
                point,
                normal,
                [0.0, 0.0],
                [0.0, 1.0],
                [1.0, 0.0, 0.0, 1.0],
                [color.x, color.y, color.z, 1.0],
            ));
        }
        indices.extend([base, base + 1, base + 2, base, base + 2, base + 3]);
    }
}

pub fn block_at(world: &VoxelWorld, position: IVec3) -> BlockKind {
    if position.y < theme::FLOOR_Y {
        return BlockKind::Bedrock;
    }
    BlockKind::from_index(nightshade_voxels::queries::block_at(world, position))
}

pub fn player_box(position: Vec3) -> (Vec3, Vec3) {
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

pub fn looks_at(eye: Vec3, forward: Vec3, point: Vec3, reach: f32, cosine: f32) -> bool {
    let offset = point - eye;
    let distance = offset.norm();
    distance > 0.01 && distance <= reach && offset.dot(&forward) / distance >= cosine
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
    villages_near(generation, x, z).any(|village| {
        village.buildings.iter().any(|footprint| {
            x >= footprint.x - 2
                && x <= footprint.x + footprint.width + 1
                && z >= footprint.z - 2
                && z <= footprint.z + footprint.depth + 1
        }) || village.roads.contains(&(x, z))
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
            .buildings
            .iter()
            .find(|footprint| {
                x >= footprint.x - 2
                    && x <= footprint.x + footprint.width + 1
                    && z >= footprint.z - 2
                    && z <= footprint.z + footprint.depth + 1
            })
            .map(|footprint| footprint.base)
    })
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

pub fn dungeon_standing(sheet: &Sheet, index: u32) -> (SpriteRegion, f32) {
    let tile = sheet
        .dungeon_tiles
        .cell(theme::SHEET_COLUMNS, theme::SHEET_ROWS, index);
    let empty = sheet.dungeon_rows.get(index as usize).copied().unwrap_or(0) as f32;
    standing_region(tile, empty)
}

fn standing_region(tile: SpriteRegion, empty: f32) -> (SpriteRegion, f32) {
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
    let mut most = 0.0f32;
    for village in villages_near(generation, x, z) {
        for pad in &village.buildings {
            let across = (pad.x - 2 - x).max(x - (pad.x + pad.width + 1)).max(0);
            let along = (pad.z - 2 - z).max(z - (pad.z + pad.depth + 1)).max(0);
            let reach = across.max(along) as f32;
            if reach >= theme::APRON {
                continue;
            }
            let share = 1.0 - reach / theme::APRON;
            most = most.max(share * share * (3.0 - 2.0 * share));
        }
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

pub fn empty_rows_below(pixels: &[u8], width: u32) -> [u8; theme::SHEET_TILES] {
    let mut empty = [0u8; theme::SHEET_TILES];
    for (index, slot) in empty.iter_mut().enumerate() {
        let column = index as u32 % theme::SHEET_COLUMNS;
        let row = index as u32 / theme::SHEET_COLUMNS;
        let mut count = 0u8;
        for texel_row in (0..theme::TILE_TEXELS).rev() {
            let y = row * theme::TILE_TEXELS + texel_row;
            let clear = (0..theme::TILE_TEXELS).all(|texel_column| {
                let x = column * theme::TILE_TEXELS + texel_column;
                pixels[((y * width + x) * 4 + 3) as usize] == 0
            });
            if !clear {
                break;
            }
            count += 1;
        }
        *slot = count.min(15);
    }
    empty
}
