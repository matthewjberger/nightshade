use crate::theme;
use nightshade_voxels::prelude::{BlockLook, Blocks, Cell};

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Block {
    Air,
    Ground,
    Concrete,
    Brick,
    Wood,
    Glass,
    Roof,
}

impl Block {
    pub const ALL: [Block; 7] = [
        Block::Air,
        Block::Ground,
        Block::Concrete,
        Block::Brick,
        Block::Wood,
        Block::Glass,
        Block::Roof,
    ];

    pub fn index(self) -> u8 {
        self as u8
    }

    fn look(self) -> BlockLook {
        let solid = |cell: Cell, falls: bool| BlockLook {
            top: cell,
            bottom: cell,
            side: cell,
            falls,
            ..Default::default()
        };
        match self {
            Block::Air => BlockLook {
                occludes: false,
                solid: false,
                replaceable: true,
                ..Default::default()
            },
            Block::Ground => solid(GROUND, false),
            Block::Concrete => solid(CONCRETE, true),
            Block::Brick => solid(BRICK, true),
            Block::Wood => solid(WOOD, false),
            Block::Glass => BlockLook {
                cutout: true,
                occludes: false,
                ..solid(GLASS, true)
            },
            Block::Roof => solid(ROOF, false),
        }
    }
}

pub const GROUND: Cell = (0, 0);
pub const CONCRETE: Cell = (1, 0);
pub const BRICK: Cell = (2, 0);
pub const WOOD: Cell = (3, 0);
pub const GLASS: Cell = (0, 1);
pub const ROOF: Cell = (1, 1);

pub fn block_table() -> Blocks {
    Blocks {
        looks: Block::ALL.iter().map(|block| block.look()).collect(),
        atlas_columns: theme::ATLAS_COLUMNS,
        atlas_rows: theme::ATLAS_ROWS,
        liquids: Vec::new(),
    }
}

pub fn building_block(x: i32, y: i32, z: i32) -> Option<Block> {
    let half_width = theme::WIDTH / 2;
    let half_depth = theme::DEPTH / 2;
    if x < -half_width
        || x >= half_width
        || z < -half_depth
        || z >= half_depth
        || !(1..=theme::HEIGHT).contains(&y)
    {
        return None;
    }
    if y == theme::HEIGHT {
        return Some(Block::Roof);
    }
    let storey_y = (y - 1) % theme::STOREY;
    let on_x_wall = x < -half_width + theme::WALL || x >= half_width - theme::WALL;
    let on_z_wall = z < -half_depth + theme::WALL || z >= half_depth - theme::WALL;
    if on_x_wall || on_z_wall {
        let along = if on_x_wall {
            z + half_depth
        } else {
            x + half_width
        };
        let pane = along % theme::WINDOW_SPAN;
        let frame = (theme::WINDOW_SPAN - theme::WINDOW_GLASS) / 2;
        let window = !(on_x_wall && on_z_wall)
            && (theme::WINDOW_LOW..theme::WINDOW_HIGH).contains(&storey_y)
            && pane >= frame
            && pane < frame + theme::WINDOW_GLASS;
        return Some(if window { Block::Glass } else { Block::Brick });
    }
    if storey_y == 0 {
        return Some(if y == 1 { Block::Concrete } else { Block::Wood });
    }
    let pillar = (x + half_width) % theme::PILLAR_SPACING < theme::PILLAR
        && (z + half_depth) % theme::PILLAR_SPACING < theme::PILLAR;
    pillar.then_some(Block::Concrete)
}

pub fn atlas_pixels() -> Vec<u8> {
    let tiles = [
        (GROUND, theme::GROUND_TILE),
        (CONCRETE, theme::CONCRETE_TILE),
        (BRICK, theme::BRICK_TILE),
        (WOOD, theme::WOOD_TILE),
        (GLASS, theme::GLASS_TILE),
        (ROOF, theme::ROOF_TILE),
    ];
    let width = theme::ATLAS_COLUMNS * theme::TILE_TEXELS;
    let height = theme::ATLAS_ROWS * theme::TILE_TEXELS;
    let mut pixels = vec![0; (width * height * 4) as usize];
    for (cell, color) in tiles {
        for row in 0..theme::TILE_TEXELS {
            for column in 0..theme::TILE_TEXELS {
                let x = cell.0 * theme::TILE_TEXELS + column;
                let y = cell.1 * theme::TILE_TEXELS + row;
                let speckle = (x.wrapping_mul(73).wrapping_add(y.wrapping_mul(151)) % 29) as i32
                    * 2
                    * theme::SPECKLE
                    / 29
                    - theme::SPECKLE;
                let at = ((y * width + x) * 4) as usize;
                for channel in 0..3 {
                    pixels[at + channel] = (color[channel] as i32 + speckle).clamp(0, 255) as u8;
                }
                let edge = theme::TILE_TEXELS - theme::FRAME_TEXELS;
                let inside = column >= theme::FRAME_TEXELS
                    && column < edge
                    && row >= theme::FRAME_TEXELS
                    && row < edge;
                pixels[at + 3] = if cell == GLASS && inside { 0 } else { 255 };
            }
        }
    }
    pixels
}
