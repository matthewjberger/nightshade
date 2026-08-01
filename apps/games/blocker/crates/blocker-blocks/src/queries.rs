use crate::data::blocks::{BlockKind, CRACK_STAGES, Cell, Palette};
use crate::data::items::{CHEST_ART, Item};
use crate::resources::{Sheet, VoxelWorld};
use crate::theme;
use nalgebra_glm::{IVec3, Vec2, Vec3, vec2};
use nightshade_render::data::decoded_image::decode_to_rgba8;
use nightshade_render::data::geometry::{Mesh, Vertex};
use nightshade_sprite::prelude::SpriteRegion;
use nightshade_voxels::prelude::Blocks;
pub use nightshade_voxels::queries::{
    block_index, box_in_liquid, chunks_of, column_of, column_settled, cube_mesh, facing_of,
    ground_height, is_solid_at, liquid_near, liquid_surface_near, raycast, raycast_to_water,
    slide_box, walkable, with_bounds,
};

pub fn footing(world: &VoxelWorld, x: i32, z: i32, near_y: i32) -> Option<i32> {
    let high = near_y + theme::FOOTING_RISE;
    let low = (near_y - theme::FOOTING_DROP).max(nightshade_voxels::data::FLOOR_Y);
    (low..=high).rev().find(|&y| {
        is_solid_at(world, IVec3::new(x, y, z))
            && !is_solid_at(world, IVec3::new(x, y + 1, z))
            && !is_solid_at(world, IVec3::new(x, y + 2, z))
    })
}

pub fn step_holds(world: &VoxelWorld, from: Vec2, to: Vec2, near_y: i32) -> bool {
    let here = footing(world, from.x.floor() as i32, from.y.floor() as i32, near_y);
    let there = footing(world, to.x.floor() as i32, to.y.floor() as i32, near_y);
    matches!((here, there), (Some(from), Some(to)) if (from - to).abs() <= theme::STEP_RISE)
}

pub fn cell_index(cell: Cell) -> f32 {
    (cell.1 * crate::data::atlas::ATLAS_COLUMNS + cell.0) as f32
}

pub fn cell_uv_exact(cell: Cell) -> (Vec2, Vec2) {
    let width = 1.0 / crate::data::atlas::ATLAS_COLUMNS as f32;
    let height = 1.0 / crate::data::atlas::ATLAS_ROWS as f32;
    (
        vec2(cell.0 as f32 * width, cell.1 as f32 * height),
        vec2((cell.0 + 1) as f32 * width, (cell.1 + 1) as f32 * height),
    )
}

pub fn art_color(character: char) -> Option<[f32; 4]> {
    match character {
        'b' => Some([0.55, 0.36, 0.2, 1.0]),
        'B' => Some([0.36, 0.22, 0.12, 1.0]),
        'i' => Some([0.86, 0.86, 0.88, 1.0]),
        'I' => Some([0.6, 0.6, 0.64, 1.0]),
        'd' => Some([0.4, 0.4, 0.45, 1.0]),
        'w' => Some([1.0, 1.0, 1.0, 1.0]),
        'r' => Some([0.85, 0.2, 0.18, 1.0]),
        'R' => Some([0.6, 0.1, 0.1, 1.0]),
        'o' => Some([0.95, 0.55, 0.15, 1.0]),
        'y' => Some([0.95, 0.85, 0.3, 1.0]),
        'Y' => Some([0.75, 0.6, 0.15, 1.0]),
        'g' => Some([0.35, 0.75, 0.3, 1.0]),
        'G' => Some([0.2, 0.5, 0.2, 1.0]),
        't' => Some([0.8, 0.65, 0.4, 1.0]),
        'T' => Some([0.6, 0.45, 0.25, 1.0]),
        'u' => Some([0.35, 0.55, 0.9, 1.0]),
        _ => None,
    }
}

pub fn decode_atlas(palette: Palette) -> (Vec<u8>, u32, u32) {
    let decoded = decode_to_rgba8(palette.bytes()).expect("the block atlas decodes");
    let (width, height) = (decoded.width, decoded.height);
    let mut pixels = decoded.rgba;
    for (cell, tint) in palette.tinted_cells().iter().copied() {
        for row in 0..crate::data::atlas::TILE_TEXELS {
            for column in 0..crate::data::atlas::TILE_TEXELS {
                let x = cell.0 * crate::data::atlas::TILE_TEXELS + column;
                let y = cell.1 * crate::data::atlas::TILE_TEXELS + row;
                let offset = ((y * width + x) * 4) as usize;
                for channel in 0..3 {
                    let value = pixels[offset + channel] as f32 * tint[channel];
                    pixels[offset + channel] = value.round().clamp(0.0, 255.0) as u8;
                }
            }
        }
    }
    for (cell, art) in CHEST_ART {
        for (row, line) in art.iter().enumerate() {
            for (column, mark) in line.chars().enumerate() {
                let Some(color) = art_color(mark) else {
                    continue;
                };
                let x = cell.0 * crate::data::atlas::TILE_TEXELS + column as u32;
                let y = cell.1 * crate::data::atlas::TILE_TEXELS + row as u32;
                let offset = ((y * width + x) * 4) as usize;
                for channel in 0..3 {
                    pixels[offset + channel] = (color[channel] * 255.0).round() as u8;
                }
                pixels[offset + 3] = 255;
            }
        }
    }
    (pixels, width, height)
}

pub fn block_at(world: &VoxelWorld, position: IVec3) -> BlockKind {
    if position.y < nightshade_voxels::data::FLOOR_Y {
        return BlockKind::Bedrock;
    }
    BlockKind::from_index(nightshade_voxels::queries::block_at(world, position))
}

pub fn item_value(item: Item) -> u32 {
    match item {
        Item::Block(kind) => match kind {
            BlockKind::DiamondOre => 30,
            BlockKind::GoldOre => 14,
            BlockKind::IronOre => 8,
            BlockKind::CoalOre => 4,
            BlockKind::Glowstone | BlockKind::Obsidian => 6,
            BlockKind::Bricks | BlockKind::Glass => 3,
            _ => 1,
        },
        Item::Tool(_) => 18,
        Item::Boat => 6,
        Item::Crop(_) => 3,
        Item::Seed(_) => 1,
        Item::WaterBucket => 4,
        Item::Bucket => 3,
        Item::Firework => 4,
    }
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

pub fn crack_stage(progress: f32) -> usize {
    ((progress.clamp(0.0, 0.999) * crate::data::blocks::CRACK_STAGES.len() as f32) as usize)
        .min(crate::data::blocks::CRACK_STAGES.len() - 1)
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
    let share = 1.0 - empty / crate::data::atlas::TILE_TEXELS as f32;
    let region = SpriteRegion::new(
        tile.uv_min,
        Vec2::new(
            tile.uv_max.x,
            tile.uv_min.y + (tile.uv_max.y - tile.uv_min.y) * share,
        ),
    );
    (region, share)
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

pub fn extruded_cell_mesh(pixels: &[u8], width: u32, cell: Cell) -> Mesh {
    let texel = 1.0 / 16.0;
    let half_depth = texel * theme::ITEM_DEPTH * 0.5;
    let alpha_at = |column: i32, row: i32| -> u8 {
        if !(0..16).contains(&column) || !(0..16).contains(&row) {
            return 0;
        }
        let x = cell.0 * crate::data::atlas::TILE_TEXELS + column as u32;
        let y = cell.1 * crate::data::atlas::TILE_TEXELS + row as u32;
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
                (column as f32 + 0.5) / crate::data::atlas::TILE_TEXELS as f32,
                (row as f32 + 0.5) / crate::data::atlas::TILE_TEXELS as f32,
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

pub fn empty_rows_below(pixels: &[u8], width: u32) -> [u8; theme::SHEET_TILES] {
    let mut empty = [0u8; theme::SHEET_TILES];
    for (index, slot) in empty.iter_mut().enumerate() {
        let column = index as u32 % theme::SHEET_COLUMNS;
        let row = index as u32 / theme::SHEET_COLUMNS;
        let mut count = 0u8;
        for texel_row in (0..crate::data::atlas::TILE_TEXELS).rev() {
            let y = row * crate::data::atlas::TILE_TEXELS + texel_row;
            let clear = (0..crate::data::atlas::TILE_TEXELS).all(|texel_column| {
                let x = column * crate::data::atlas::TILE_TEXELS + texel_column;
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
