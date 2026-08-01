use crate::theme;
use nightshade_voxels::prelude::{BlockLook, Blocks, Cell, Fill, Liquid, Shape};

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Block {
    Air,
    Stone,
    Dirt,
    Grass,
    Sand,
    Obsidian,
    WaterOne,
    WaterTwo,
    WaterThree,
    WaterFour,
    WaterFive,
    LavaOne,
    LavaTwo,
    LavaThree,
    LavaFour,
    LavaFive,
}

impl Block {
    pub const ALL: [Block; 16] = [
        Block::Air,
        Block::Stone,
        Block::Dirt,
        Block::Grass,
        Block::Sand,
        Block::Obsidian,
        Block::WaterOne,
        Block::WaterTwo,
        Block::WaterThree,
        Block::WaterFour,
        Block::WaterFive,
        Block::LavaOne,
        Block::LavaTwo,
        Block::LavaThree,
        Block::LavaFour,
        Block::LavaFive,
    ];

    pub fn index(self) -> u8 {
        self as u8
    }

    fn fill(self) -> Option<Fill> {
        let (liquid, level) = match self {
            Block::WaterOne => (theme::WATER, 1),
            Block::WaterTwo => (theme::WATER, 2),
            Block::WaterThree => (theme::WATER, 3),
            Block::WaterFour => (theme::WATER, 4),
            Block::WaterFive => (theme::WATER, 5),
            Block::LavaOne => (theme::LAVA, 1),
            Block::LavaTwo => (theme::LAVA, 2),
            Block::LavaThree => (theme::LAVA, 3),
            Block::LavaFour => (theme::LAVA, 4),
            Block::LavaFive => (theme::LAVA, 5),
            _ => return None,
        };
        Some(Fill { liquid, level })
    }

    fn look(self) -> BlockLook {
        let solid = |top: Cell, bottom: Cell, side: Cell| BlockLook {
            top,
            bottom,
            side,
            ..Default::default()
        };
        match self {
            Block::Air => BlockLook {
                occludes: false,
                solid: false,
                replaceable: true,
                ..Default::default()
            },
            Block::Stone => solid(STONE, STONE, STONE),
            Block::Dirt => solid(DIRT, DIRT, DIRT),
            Block::Grass => solid(GRASS_TOP, DIRT, GRASS_SIDE),
            Block::Sand => BlockLook {
                falls: true,
                ..solid(SAND, SAND, SAND)
            },
            Block::Obsidian => solid(OBSIDIAN, OBSIDIAN, OBSIDIAN),
            _ => {
                let fill = self.fill();
                let cell = if fill.is_some_and(|fill| fill.liquid == theme::LAVA) {
                    LAVA
                } else {
                    WATER
                };
                BlockLook {
                    top: cell,
                    bottom: cell,
                    side: cell,
                    shape: Shape::Liquid,
                    occludes: false,
                    solid: false,
                    replaceable: true,
                    fill,
                    ..Default::default()
                }
            }
        }
    }
}

pub const STONE: Cell = (0, 0);
pub const DIRT: Cell = (1, 0);
pub const GRASS_TOP: Cell = (2, 0);
pub const GRASS_SIDE: Cell = (3, 0);
pub const SAND: Cell = (0, 1);
pub const WATER: Cell = (1, 1);
pub const LAVA: Cell = (2, 1);
pub const OBSIDIAN: Cell = (3, 1);

pub fn block_table() -> Blocks {
    Blocks {
        looks: Block::ALL.iter().map(|block| block.look()).collect(),
        atlas_columns: theme::ATLAS_COLUMNS,
        atlas_rows: theme::ATLAS_ROWS,
        liquids: vec![
            Liquid {
                levels: [
                    Block::Air,
                    Block::WaterOne,
                    Block::WaterTwo,
                    Block::WaterThree,
                    Block::WaterFour,
                    Block::WaterFive,
                ]
                .map(Block::index),
                reach: theme::WATER_REACH,
                pace: 1,
                material: theme::WATER_MATERIAL.to_string(),
            },
            Liquid {
                levels: [
                    Block::Air,
                    Block::LavaOne,
                    Block::LavaTwo,
                    Block::LavaThree,
                    Block::LavaFour,
                    Block::LavaFive,
                ]
                .map(Block::index),
                reach: theme::LAVA_REACH,
                pace: theme::LAVA_PACE,
                material: theme::LAVA_MATERIAL.to_string(),
            },
        ],
    }
}

pub fn atlas_pixels() -> Vec<u8> {
    let tiles = [
        (STONE, theme::STONE_TILE),
        (DIRT, theme::DIRT_TILE),
        (GRASS_TOP, theme::GRASS_TOP_TILE),
        (GRASS_SIDE, theme::GRASS_SIDE_TILE),
        (SAND, theme::SAND_TILE),
        (WATER, theme::WATER_TILE),
        (LAVA, theme::LAVA_TILE),
        (OBSIDIAN, theme::OBSIDIAN_TILE),
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
                pixels[at + 3] = 255;
            }
        }
    }
    pixels
}
