use serde::{Deserialize, Serialize};
pub mod abilities;
pub mod assets;
pub mod chatter;
pub mod folk;
pub mod places;
pub mod quests;
pub mod sounds;
pub mod spawns;

pub use abilities::*;
pub use folk::*;
pub use places::*;
pub use quests::*;
pub use sounds::*;
pub use spawns::*;

use crate::theme;
use nalgebra_glm::{IVec3, Vec2, Vec3, vec2};
use nightshade_voxels::prelude::{BlockLook, Blocks, Fill, Liquid, Part, Patch};
pub use nightshade_voxels::prelude::{Cell, Face, Shape};

pub fn cell_index(cell: Cell) -> f32 {
    (cell.1 * theme::ATLAS_COLUMNS + cell.0) as f32
}

pub fn cell_uv_exact(cell: Cell) -> (Vec2, Vec2) {
    let width = 1.0 / theme::ATLAS_COLUMNS as f32;
    let height = 1.0 / theme::ATLAS_ROWS as f32;
    (
        vec2(cell.0 as f32 * width, cell.1 as f32 * height),
        vec2((cell.0 + 1) as f32 * width, (cell.1 + 1) as f32 * height),
    )
}

pub fn block_table() -> Blocks {
    let looks = BLOCKS
        .iter()
        .map(|info| {
            let kind = &info.kind;
            BlockLook {
                top: info.top,
                bottom: info.bottom,
                side: info.side,
                shape: info.shape,
                cutout: info.cutout,
                transparent: info.transparent,
                occludes: *kind != BlockKind::Air
                    && info.shape == Shape::Cube
                    && !info.cutout
                    && !info.transparent,
                fill: water_fill(*kind),
                parts: kind.parts(),
                stacked: kind.door_facing().is_some().then_some(DOOR_UPPER),
                tall: kind.door_facing().is_some(),
                support: kind.support(),
                swung: kind.swung().map(BlockKind::index),
                marked: kind.glow().is_some() || *kind == BlockKind::Signpost,
                solid: info.solid,
                falls: matches!(kind, BlockKind::Sand | BlockKind::Gravel),
                replaceable: water_level(*kind) > 0
                    || matches!(
                        kind,
                        BlockKind::Air
                            | BlockKind::RedFlower
                            | BlockKind::YellowFlower
                            | BlockKind::DeadBush
                    ),
            }
        })
        .collect();
    Blocks {
        looks,
        atlas_columns: theme::ATLAS_COLUMNS,
        atlas_rows: theme::ATLAS_ROWS,
        liquids: vec![Liquid {
            levels: [
                BlockKind::Air,
                BlockKind::WaterOne,
                BlockKind::WaterTwo,
                BlockKind::WaterThree,
                BlockKind::WaterFour,
                BlockKind::Water,
            ]
            .map(BlockKind::index),
            reach: theme::WATER_RUN,
            pace: 1,
            material: theme::WATER_MATERIAL.to_string(),
        }],
    }
}

fn water_fill(kind: BlockKind) -> Option<Fill> {
    let level = water_level(kind);
    (level > 0).then_some(Fill {
        liquid: theme::WATER,
        level,
    })
}

fn water_level(kind: BlockKind) -> u8 {
    match kind {
        BlockKind::Water => 5,
        BlockKind::WaterFour => 4,
        BlockKind::WaterThree => 3,
        BlockKind::WaterTwo => 2,
        BlockKind::WaterOne => 1,
        _ => 0,
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
#[repr(u8)]
pub enum BlockKind {
    #[default]
    Air,
    Bedrock,
    Stone,
    Dirt,
    Grass,
    Cobblestone,
    Sand,
    Gravel,
    OakLog,
    OakLeaves,
    OakPlanks,
    Glass,
    Bricks,
    Glowstone,
    CoalOre,
    IronOre,
    GoldOre,
    DiamondOre,
    CraftingTable,
    Tnt,
    Obsidian,
    WoolRed,
    WoolBlue,
    WoolWhite,
    RedFlower,
    YellowFlower,
    Water,
    Campfire,
    Signpost,
    Torch,
    TorchNorth,
    TorchSouth,
    TorchEast,
    TorchWest,
    DoorNorth,
    DoorSouth,
    DoorEast,
    DoorWest,
    DoorOpenNorth,
    DoorOpenSouth,
    DoorOpenEast,
    DoorOpenWest,
    WaterFour,
    WaterThree,
    WaterTwo,
    WaterOne,
    Chest,
    Snow,
    SnowGrass,
    Ice,
    RedSand,
    Terracotta,
    DeadBush,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ToolKind {
    Pickaxe,
    Shovel,
    Axe,
    Sword,
}

impl ToolKind {
    pub fn index(self) -> u8 {
        self as u8
    }

    pub fn from_index(index: u8) -> Option<ToolKind> {
        TOOLS.get(index as usize).map(|row| row.kind)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct BlockInfo {
    pub kind: BlockKind,
    pub door: Option<(IVec3, bool)>,
    pub lean: Vec3,
    pub on_wall: bool,
    pub glow: Option<(Vec3, f32, f32)>,
    pub grows_crops: bool,
    pub parts: &'static [Part],
    pub name: &'static str,
    pub top: Cell,
    pub bottom: Cell,
    pub side: Cell,
    pub shape: Shape,
    pub cutout: bool,
    pub transparent: bool,
    pub solid: bool,
    pub hardness: f32,
    pub tool: Option<ToolKind>,
    pub drop: Option<BlockKind>,
}

pub const BLOCKS: [BlockInfo; 53] = [
    BlockInfo {
        kind: BlockKind::Air,
        door: None,
        lean: Vec3::new(0.0, 0.0, 0.0),
        on_wall: false,
        glow: None,
        grows_crops: false,
        parts: &[],
        name: "Air",
        top: (0, 0),
        bottom: (0, 0),
        side: (0, 0),
        shape: Shape::Cube,
        cutout: false,
        transparent: false,
        solid: false,
        hardness: 0.0,
        tool: None,
        drop: None,
    },
    BlockInfo {
        kind: BlockKind::Bedrock,
        ..uniform("Bedrock", (4, 3), f32::INFINITY, None)
    },
    BlockInfo {
        kind: BlockKind::Stone,
        drop: Some(BlockKind::Cobblestone),
        ..uniform("Stone", (19, 6), 1.5, Some(ToolKind::Pickaxe))
    },
    BlockInfo {
        kind: BlockKind::Dirt,
        grows_crops: true,
        ..uniform("Dirt", (8, 4), 0.5, Some(ToolKind::Shovel))
    },
    BlockInfo {
        kind: BlockKind::Grass,
        grows_crops: true,
        drop: Some(BlockKind::Dirt),
        ..cube(
            "Grass",
            (3, 10),
            (8, 4),
            (0, 10),
            0.6,
            Some(ToolKind::Shovel),
        )
    },
    BlockInfo {
        kind: BlockKind::Cobblestone,
        ..uniform("Cobblestone", (2, 5), 2.0, Some(ToolKind::Pickaxe))
    },
    BlockInfo {
        kind: BlockKind::Sand,
        ..uniform("Sand", (18, 7), 0.5, Some(ToolKind::Shovel))
    },
    BlockInfo {
        kind: BlockKind::Gravel,
        ..uniform("Gravel", (4, 10), 0.6, Some(ToolKind::Shovel))
    },
    BlockInfo {
        kind: BlockKind::OakLog,
        ..cube(
            "Oak Log",
            (13, 13),
            (13, 13),
            (12, 13),
            2.0,
            Some(ToolKind::Axe),
        )
    },
    BlockInfo {
        kind: BlockKind::OakLeaves,
        cutout: true,
        ..uniform("Oak Leaves", (15, 12), 0.2, None)
    },
    BlockInfo {
        kind: BlockKind::OakPlanks,
        ..uniform("Oak Planks", (12, 15), 2.0, Some(ToolKind::Axe))
    },
    BlockInfo {
        kind: BlockKind::Glass,
        transparent: true,
        drop: Some(BlockKind::Air),
        ..uniform("Glass", (9, 2), 0.3, None)
    },
    BlockInfo {
        kind: BlockKind::Bricks,
        ..uniform("Bricks", (5, 3), 2.0, Some(ToolKind::Pickaxe))
    },
    BlockInfo {
        kind: BlockKind::Glowstone,
        ..uniform("Glowstone", (13, 9), 0.3, None)
    },
    BlockInfo {
        kind: BlockKind::CoalOre,
        ..uniform("Coal Ore", (1, 5), 3.0, Some(ToolKind::Pickaxe))
    },
    BlockInfo {
        kind: BlockKind::IronOre,
        ..uniform("Iron Ore", (15, 11), 3.0, Some(ToolKind::Pickaxe))
    },
    BlockInfo {
        kind: BlockKind::GoldOre,
        ..uniform("Gold Ore", (15, 9), 3.0, Some(ToolKind::Pickaxe))
    },
    BlockInfo {
        kind: BlockKind::DiamondOre,
        ..uniform("Diamond Ore", (8, 3), 3.0, Some(ToolKind::Pickaxe))
    },
    BlockInfo {
        kind: BlockKind::CraftingTable,
        ..cube(
            "Crafting Table",
            (4, 6),
            (12, 15),
            (12, 15),
            2.5,
            Some(ToolKind::Axe),
        )
    },
    BlockInfo {
        kind: BlockKind::Tnt,
        ..cube("TNT", (19, 13), (19, 14), (19, 15), 0.0, None)
    },
    BlockInfo {
        kind: BlockKind::Obsidian,
        ..uniform("Obsidian", (4, 15), 12.0, Some(ToolKind::Pickaxe))
    },
    BlockInfo {
        kind: BlockKind::WoolRed,
        ..uniform("Red Wool", (21, 13), 0.8, Some(ToolKind::Sword))
    },
    BlockInfo {
        kind: BlockKind::WoolBlue,
        ..uniform("Blue Wool", (21, 2), 0.8, Some(ToolKind::Sword))
    },
    BlockInfo {
        kind: BlockKind::WoolWhite,
        ..uniform("White Wool", (21, 15), 0.8, Some(ToolKind::Sword))
    },
    BlockInfo {
        kind: BlockKind::RedFlower,
        ..plant("Rose", (14, 2), 0.0)
    },
    BlockInfo {
        kind: BlockKind::YellowFlower,
        ..plant("Dandelion", (13, 6), 0.0)
    },
    BlockInfo {
        kind: BlockKind::Water,
        door: None,
        lean: Vec3::new(0.0, 0.0, 0.0),
        on_wall: false,
        glow: None,
        grows_crops: false,
        parts: &[],
        name: "Water",
        top: (20, 6),
        bottom: (20, 6),
        side: (20, 6),
        shape: Shape::Liquid,
        cutout: false,
        transparent: false,
        solid: false,
        hardness: f32::INFINITY,
        tool: None,
        drop: None,
    },
    BlockInfo {
        kind: BlockKind::Campfire,
        glow: Some((
            theme::CAMPFIRE_COLOR,
            theme::CAMPFIRE_STRENGTH,
            theme::CAMPFIRE_RANGE,
        )),
        ..cube(
            "Campfire",
            (0, 0),
            (2, 5),
            (2, 5),
            1.0,
            Some(ToolKind::Pickaxe),
        )
    },
    BlockInfo {
        kind: BlockKind::Signpost,
        parts: &SIGN_PARTS,
        shape: Shape::Parts,
        ..plant("Signpost", (12, 13), 0.5)
    },
    BlockInfo {
        kind: BlockKind::Torch,
        glow: Some((
            theme::TORCH_COLOR,
            theme::TORCH_STRENGTH,
            theme::TORCH_RANGE,
        )),
        parts: &TORCH_PARTS,
        shape: Shape::Parts,
        drop: Some(BlockKind::Torch),
        ..plant("Torch", (20, 1), 0.0)
    },
    BlockInfo {
        kind: BlockKind::TorchNorth,
        lean: Vec3::new(0.0, 0.0, -1.0),
        on_wall: true,
        glow: Some((
            theme::TORCH_COLOR,
            theme::TORCH_STRENGTH,
            theme::TORCH_RANGE,
        )),
        parts: &TORCH_NORTH_PARTS,
        shape: Shape::Parts,
        drop: Some(BlockKind::Torch),
        ..plant("Torch", (20, 1), 0.0)
    },
    BlockInfo {
        kind: BlockKind::TorchSouth,
        lean: Vec3::new(0.0, 0.0, 1.0),
        on_wall: true,
        glow: Some((
            theme::TORCH_COLOR,
            theme::TORCH_STRENGTH,
            theme::TORCH_RANGE,
        )),
        parts: &TORCH_SOUTH_PARTS,
        shape: Shape::Parts,
        drop: Some(BlockKind::Torch),
        ..plant("Torch", (20, 1), 0.0)
    },
    BlockInfo {
        kind: BlockKind::TorchEast,
        lean: Vec3::new(1.0, 0.0, 0.0),
        on_wall: true,
        glow: Some((
            theme::TORCH_COLOR,
            theme::TORCH_STRENGTH,
            theme::TORCH_RANGE,
        )),
        parts: &TORCH_EAST_PARTS,
        shape: Shape::Parts,
        drop: Some(BlockKind::Torch),
        ..plant("Torch", (20, 1), 0.0)
    },
    BlockInfo {
        kind: BlockKind::TorchWest,
        lean: Vec3::new(-1.0, 0.0, 0.0),
        on_wall: true,
        glow: Some((
            theme::TORCH_COLOR,
            theme::TORCH_STRENGTH,
            theme::TORCH_RANGE,
        )),
        parts: &TORCH_WEST_PARTS,
        shape: Shape::Parts,
        drop: Some(BlockKind::Torch),
        ..plant("Torch", (20, 1), 0.0)
    },
    BlockInfo {
        kind: BlockKind::DoorNorth,
        door: Some((IVec3::new(0, 0, -1), false)),
        parts: &DOOR_NORTH_PARTS,
        shape: Shape::Parts,
        solid: true,
        cutout: true,
        drop: Some(BlockKind::DoorNorth),
        ..cube(
            "Door",
            DOOR_LOWER,
            DOOR_LOWER,
            DOOR_LOWER,
            2.0,
            Some(ToolKind::Axe),
        )
    },
    BlockInfo {
        kind: BlockKind::DoorSouth,
        door: Some((IVec3::new(0, 0, 1), false)),
        parts: &DOOR_SOUTH_PARTS,
        shape: Shape::Parts,
        solid: true,
        cutout: true,
        drop: Some(BlockKind::DoorNorth),
        ..cube(
            "Door",
            DOOR_LOWER,
            DOOR_LOWER,
            DOOR_LOWER,
            2.0,
            Some(ToolKind::Axe),
        )
    },
    BlockInfo {
        kind: BlockKind::DoorEast,
        door: Some((IVec3::new(1, 0, 0), false)),
        parts: &DOOR_EAST_PARTS,
        shape: Shape::Parts,
        solid: true,
        cutout: true,
        drop: Some(BlockKind::DoorNorth),
        ..cube(
            "Door",
            DOOR_LOWER,
            DOOR_LOWER,
            DOOR_LOWER,
            2.0,
            Some(ToolKind::Axe),
        )
    },
    BlockInfo {
        kind: BlockKind::DoorWest,
        door: Some((IVec3::new(-1, 0, 0), false)),
        parts: &DOOR_WEST_PARTS,
        shape: Shape::Parts,
        solid: true,
        cutout: true,
        drop: Some(BlockKind::DoorNorth),
        ..cube(
            "Door",
            DOOR_LOWER,
            DOOR_LOWER,
            DOOR_LOWER,
            2.0,
            Some(ToolKind::Axe),
        )
    },
    BlockInfo {
        kind: BlockKind::DoorOpenNorth,
        door: Some((IVec3::new(0, 0, -1), true)),
        parts: &DOOR_OPEN_NORTH_PARTS,
        shape: Shape::Parts,
        solid: false,
        cutout: true,
        drop: Some(BlockKind::DoorNorth),
        ..cube(
            "Door",
            DOOR_LOWER,
            DOOR_LOWER,
            DOOR_LOWER,
            2.0,
            Some(ToolKind::Axe),
        )
    },
    BlockInfo {
        kind: BlockKind::DoorOpenSouth,
        door: Some((IVec3::new(0, 0, 1), true)),
        parts: &DOOR_OPEN_SOUTH_PARTS,
        shape: Shape::Parts,
        solid: false,
        cutout: true,
        drop: Some(BlockKind::DoorNorth),
        ..cube(
            "Door",
            DOOR_LOWER,
            DOOR_LOWER,
            DOOR_LOWER,
            2.0,
            Some(ToolKind::Axe),
        )
    },
    BlockInfo {
        kind: BlockKind::DoorOpenEast,
        door: Some((IVec3::new(1, 0, 0), true)),
        parts: &DOOR_OPEN_EAST_PARTS,
        shape: Shape::Parts,
        solid: false,
        cutout: true,
        drop: Some(BlockKind::DoorNorth),
        ..cube(
            "Door",
            DOOR_LOWER,
            DOOR_LOWER,
            DOOR_LOWER,
            2.0,
            Some(ToolKind::Axe),
        )
    },
    BlockInfo {
        kind: BlockKind::DoorOpenWest,
        door: Some((IVec3::new(-1, 0, 0), true)),
        parts: &DOOR_OPEN_WEST_PARTS,
        shape: Shape::Parts,
        solid: false,
        cutout: true,
        drop: Some(BlockKind::DoorNorth),
        ..cube(
            "Door",
            DOOR_LOWER,
            DOOR_LOWER,
            DOOR_LOWER,
            2.0,
            Some(ToolKind::Axe),
        )
    },
    BlockInfo {
        kind: BlockKind::WaterFour,
        door: None,
        lean: Vec3::new(0.0, 0.0, 0.0),
        on_wall: false,
        glow: None,
        grows_crops: false,
        parts: &[],
        name: "Water",
        top: (20, 6),
        bottom: (20, 6),
        side: (20, 6),
        shape: Shape::Liquid,
        cutout: false,
        transparent: false,
        solid: false,
        hardness: f32::INFINITY,
        tool: None,
        drop: None,
    },
    BlockInfo {
        kind: BlockKind::WaterThree,
        door: None,
        lean: Vec3::new(0.0, 0.0, 0.0),
        on_wall: false,
        glow: None,
        grows_crops: false,
        parts: &[],
        name: "Water",
        top: (20, 6),
        bottom: (20, 6),
        side: (20, 6),
        shape: Shape::Liquid,
        cutout: false,
        transparent: false,
        solid: false,
        hardness: f32::INFINITY,
        tool: None,
        drop: None,
    },
    BlockInfo {
        kind: BlockKind::WaterTwo,
        door: None,
        lean: Vec3::new(0.0, 0.0, 0.0),
        on_wall: false,
        glow: None,
        grows_crops: false,
        parts: &[],
        name: "Water",
        top: (20, 6),
        bottom: (20, 6),
        side: (20, 6),
        shape: Shape::Liquid,
        cutout: false,
        transparent: false,
        solid: false,
        hardness: f32::INFINITY,
        tool: None,
        drop: None,
    },
    BlockInfo {
        kind: BlockKind::WaterOne,
        door: None,
        lean: Vec3::new(0.0, 0.0, 0.0),
        on_wall: false,
        glow: None,
        grows_crops: false,
        parts: &[],
        name: "Water",
        top: (20, 6),
        bottom: (20, 6),
        side: (20, 6),
        shape: Shape::Liquid,
        cutout: false,
        transparent: false,
        solid: false,
        hardness: f32::INFINITY,
        tool: None,
        drop: None,
    },
    BlockInfo {
        kind: BlockKind::Chest,
        ..cube(
            "Chest",
            theme::CHEST_TOP_CELL,
            theme::CHEST_TOP_CELL,
            theme::CHEST_SIDE_CELL,
            2.0,
            Some(ToolKind::Axe),
        )
    },
    BlockInfo {
        kind: BlockKind::Snow,
        ..uniform("Snow", (19, 3), 0.2, Some(ToolKind::Shovel))
    },
    BlockInfo {
        kind: BlockKind::SnowGrass,
        drop: Some(BlockKind::Dirt),
        ..cube(
            "Snowy Grass",
            (19, 3),
            (8, 4),
            (2, 10),
            0.6,
            Some(ToolKind::Shovel),
        )
    },
    BlockInfo {
        kind: BlockKind::Ice,
        drop: Some(BlockKind::Air),
        ..uniform("Ice", (15, 3), 0.5, Some(ToolKind::Pickaxe))
    },
    BlockInfo {
        kind: BlockKind::RedSand,
        ..uniform("Red Sand", (5, 10), 0.5, Some(ToolKind::Shovel))
    },
    BlockInfo {
        kind: BlockKind::Terracotta,
        ..uniform("Terracotta", (5, 11), 1.25, Some(ToolKind::Pickaxe))
    },
    BlockInfo {
        kind: BlockKind::DeadBush,
        ..plant("Dead Bush", (19, 2), 0.0)
    },
];

const fn cube(
    name: &'static str,
    top: Cell,
    bottom: Cell,
    side: Cell,
    hardness: f32,
    tool: Option<ToolKind>,
) -> BlockInfo {
    BlockInfo {
        kind: BlockKind::Air,
        door: None,
        lean: Vec3::new(0.0, 0.0, 0.0),
        on_wall: false,
        glow: None,
        grows_crops: false,
        parts: &[],
        name,
        top,
        bottom,
        side,
        shape: Shape::Cube,
        cutout: false,
        transparent: false,
        solid: true,
        hardness,
        tool,
        drop: None,
    }
}

const fn uniform(
    name: &'static str,
    cell: Cell,
    hardness: f32,
    tool: Option<ToolKind>,
) -> BlockInfo {
    cube(name, cell, cell, cell, hardness, tool)
}

const fn plant(name: &'static str, cell: Cell, hardness: f32) -> BlockInfo {
    BlockInfo {
        kind: BlockKind::Air,
        door: None,
        lean: Vec3::new(0.0, 0.0, 0.0),
        on_wall: false,
        glow: None,
        grows_crops: false,
        parts: &[],
        name,
        top: cell,
        bottom: cell,
        side: cell,
        shape: Shape::Cross,
        cutout: true,
        transparent: false,
        solid: false,
        hardness,
        tool: None,
        drop: None,
    }
}

impl BlockKind {
    pub fn from_index(index: u8) -> BlockKind {
        BLOCKS
            .get(index as usize)
            .map(|row| row.kind)
            .unwrap_or(BlockKind::Air)
    }

    pub fn index(self) -> u8 {
        self as u8
    }

    pub fn info(self) -> BlockInfo {
        BLOCKS[self as usize]
    }

    pub fn door_facing(self) -> Option<(IVec3, bool)> {
        BLOCKS[self as usize].door
    }

    pub fn door_kind(facing: IVec3, open: bool) -> BlockKind {
        match (facing.x, facing.z, open) {
            (0, -1, false) => BlockKind::DoorNorth,
            (0, 1, false) => BlockKind::DoorSouth,
            (1, 0, false) => BlockKind::DoorEast,
            (-1, 0, false) => BlockKind::DoorWest,
            (0, -1, true) => BlockKind::DoorOpenNorth,
            (0, 1, true) => BlockKind::DoorOpenSouth,
            (1, 0, true) => BlockKind::DoorOpenEast,
            _ => BlockKind::DoorOpenWest,
        }
    }

    pub fn swung(self) -> Option<BlockKind> {
        let (facing, open) = self.door_facing()?;
        Some(BlockKind::door_kind(facing, !open))
    }

    pub fn lean(self) -> Vec3 {
        BLOCKS[self as usize].lean
    }

    pub fn on_wall(self) -> bool {
        BLOCKS[self as usize].on_wall
    }

    pub fn wall_torch(normal: IVec3) -> Option<BlockKind> {
        match (normal.x, normal.y, normal.z) {
            (0, 1, 0) => Some(BlockKind::Torch),
            (0, 0, -1) => Some(BlockKind::TorchNorth),
            (0, 0, 1) => Some(BlockKind::TorchSouth),
            (1, 0, 0) => Some(BlockKind::TorchEast),
            (-1, 0, 0) => Some(BlockKind::TorchWest),
            _ => None,
        }
    }

    pub fn glow(self) -> Option<(Vec3, f32, f32)> {
        BLOCKS[self as usize].glow
    }

    pub fn grows_crops(self) -> bool {
        BLOCKS[self as usize].grows_crops
    }

    pub fn drop(self) -> Option<BlockKind> {
        match self.info().drop {
            Some(BlockKind::Air) => None,
            Some(other) => Some(other),
            None => Some(self),
        }
    }
}

impl BlockKind {
    pub fn parts(self) -> &'static [Part] {
        BLOCKS[self as usize].parts
    }

    pub fn support(self) -> Option<[i32; 3]> {
        let lean = self.lean();
        if self.on_wall() {
            return Some([-lean.x as i32, -lean.y as i32, -lean.z as i32]);
        }
        match self.info().shape {
            Shape::Cross => Some([0, -1, 0]),
            Shape::Parts if self.door_facing().is_none() => Some([0, -1, 0]),
            _ => None,
        }
    }
}

pub const DOOR_LOWER: Cell = (9, 3);
pub const DOOR_UPPER: Cell = (9, 4);

const TEXEL: f32 = 1.0 / theme::TILE_TEXELS as f32;
const PATCH_INSET: f32 = 0.25 * TEXEL;
const TORCH_HALF_WIDTH: f32 = TEXEL;
const TORCH_HEIGHT: f32 = 10.0 * TEXEL;
const DOOR_THICKNESS: f32 = 3.0 * TEXEL;

const fn patch(low_x: f32, low_y: f32, high_x: f32, high_y: f32) -> Patch {
    Patch {
        low: [low_x * TEXEL + PATCH_INSET, low_y * TEXEL + PATCH_INSET],
        high: [high_x * TEXEL - PATCH_INSET, high_y * TEXEL - PATCH_INSET],
    }
}

const fn torch(lean: [f32; 3]) -> Part {
    let wall = lean[0] != 0.0 || lean[2] != 0.0;
    let rise = if wall { theme::TORCH_WALL_RISE } else { 0.0 };
    let base_x = 0.5 - lean[0] * theme::TORCH_WALL_SHIFT;
    let base_z = 0.5 - lean[2] * theme::TORCH_WALL_SHIFT;
    Part {
        low: [base_x - TORCH_HALF_WIDTH, rise, base_z - TORCH_HALF_WIDTH],
        high: [
            base_x + TORCH_HALF_WIDTH,
            rise + TORCH_HEIGHT,
            base_z + TORCH_HALF_WIDTH,
        ],
        lift: [
            lean[0] * theme::TORCH_WALL_TILT,
            0.0,
            lean[2] * theme::TORCH_WALL_TILT,
        ],
        side: patch(7.0, 6.0, 9.0, 16.0),
        top: patch(7.0, 6.0, 9.0, 8.0),
        bottom: patch(7.0, 6.0, 9.0, 8.0),
    }
}

const fn door(facing: [i32; 3], open: bool) -> Part {
    let side = if open {
        [-facing[2], 0, facing[0]]
    } else {
        facing
    };
    let mut low = [0.0, 0.0, 0.0];
    let mut high = [1.0, 1.0, 1.0];
    if side[0] > 0 {
        low[0] = 1.0 - DOOR_THICKNESS;
    } else if side[0] < 0 {
        high[0] = DOOR_THICKNESS;
    } else if side[2] > 0 {
        low[2] = 1.0 - DOOR_THICKNESS;
    } else {
        high[2] = DOOR_THICKNESS;
    }
    Part {
        low,
        high,
        lift: [0.0, 0.0, 0.0],
        side: Patch::FULL,
        top: Patch::FULL,
        bottom: Patch::FULL,
    }
}

const NORTH: [i32; 3] = [0, 0, -1];
const SOUTH: [i32; 3] = [0, 0, 1];
const EAST: [i32; 3] = [1, 0, 0];
const WEST: [i32; 3] = [-1, 0, 0];

const TORCH_PARTS: [Part; 1] = [torch([0.0, 0.0, 0.0])];
const TORCH_NORTH_PARTS: [Part; 1] = [torch([0.0, 0.0, -1.0])];
const TORCH_SOUTH_PARTS: [Part; 1] = [torch([0.0, 0.0, 1.0])];
const TORCH_EAST_PARTS: [Part; 1] = [torch([1.0, 0.0, 0.0])];
const TORCH_WEST_PARTS: [Part; 1] = [torch([-1.0, 0.0, 0.0])];
const DOOR_NORTH_PARTS: [Part; 1] = [door(NORTH, false)];
const DOOR_SOUTH_PARTS: [Part; 1] = [door(SOUTH, false)];
const DOOR_EAST_PARTS: [Part; 1] = [door(EAST, false)];
const DOOR_WEST_PARTS: [Part; 1] = [door(WEST, false)];
const DOOR_OPEN_NORTH_PARTS: [Part; 1] = [door(NORTH, true)];
const DOOR_OPEN_SOUTH_PARTS: [Part; 1] = [door(SOUTH, true)];
const DOOR_OPEN_EAST_PARTS: [Part; 1] = [door(EAST, true)];
const DOOR_OPEN_WEST_PARTS: [Part; 1] = [door(WEST, true)];
const SIGN_PARTS: [Part; 2] = [
    Part {
        low: [7.0 * TEXEL, 0.0, 7.0 * TEXEL],
        high: [9.0 * TEXEL, 9.0 * TEXEL, 9.0 * TEXEL],
        lift: [0.0, 0.0, 0.0],
        side: patch(7.0, 7.0, 9.0, 16.0),
        top: patch(7.0, 7.0, 9.0, 16.0),
        bottom: patch(7.0, 7.0, 9.0, 16.0),
    },
    Part {
        low: [TEXEL, 9.0 * TEXEL, 7.0 * TEXEL],
        high: [15.0 * TEXEL, 1.0, 9.0 * TEXEL],
        lift: [0.0, 0.0, 0.0],
        side: patch(1.0, 0.0, 15.0, 7.0),
        top: patch(1.0, 0.0, 15.0, 7.0),
        bottom: patch(1.0, 0.0, 15.0, 7.0),
    },
];

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
#[serde(into = "u8", try_from = "u8")]
pub enum Land {
    #[default]
    Rolling,
    Flat,
    Islands,
    Peaks,
    Extreme,
}

#[derive(Clone, Copy, Debug)]
pub struct LandInfo {
    pub kind: Land,
    pub name: &'static str,
    pub told: &'static str,
    pub rock: f32,
    pub headroom: i32,
    pub lift: f32,
    pub code: u8,
}

pub const LANDS: [LandInfo; 5] = [
    LandInfo {
        kind: Land::Rolling,
        name: "Rolling",
        told: "Hills, shores and open ground",
        rock: 1.0,
        headroom: 10,
        lift: 0.55,
        code: 0,
    },
    LandInfo {
        kind: Land::Flat,
        name: "Flat",
        told: "One level plain to build on",
        rock: 0.0,
        headroom: 10,
        lift: 0.0,
        code: 1,
    },
    LandInfo {
        kind: Land::Islands,
        name: "Islands",
        told: "Small land in a wide sea",
        rock: 0.9,
        headroom: 10,
        lift: 0.6,
        code: 2,
    },
    LandInfo {
        kind: Land::Peaks,
        name: "Peaks",
        told: "Tall ground and deep valleys",
        rock: 1.5,
        headroom: 16,
        lift: 0.75,
        code: 3,
    },
    LandInfo {
        kind: Land::Extreme,
        name: "Extreme",
        told: "Cliffs, arches and stone that leans over you",
        rock: 2.0,
        headroom: 28,
        lift: 1.05,
        code: 4,
    },
];

impl Land {
    pub fn name(self) -> &'static str {
        LANDS[self as usize].name
    }

    pub fn told(self) -> &'static str {
        LANDS[self as usize].told
    }

    pub fn rock(self) -> f32 {
        LANDS[self as usize].rock
    }

    pub fn headroom(self) -> i32 {
        LANDS[self as usize].headroom
    }

    pub fn lift(self) -> f32 {
        LANDS[self as usize].lift
    }

    pub fn code(self) -> u8 {
        LANDS[self as usize].code
    }

    pub fn next(self) -> Land {
        LANDS[(self as usize + 1) % LANDS.len()].kind
    }

    pub fn from_code(code: u8) -> Option<Land> {
        LANDS.get(code as usize).map(|row| row.kind)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
#[serde(into = "u8", try_from = "u8")]
pub enum Woods {
    Bare,
    Few,
    #[default]
    Normal,
    Thick,
}

#[derive(Clone, Copy, Debug)]
pub struct WoodsInfo {
    pub kind: Woods,
    pub name: &'static str,
    pub scale: f32,
    pub code: u8,
}

pub const WOODS: [WoodsInfo; 4] = [
    WoodsInfo {
        kind: Woods::Bare,
        name: "None",
        scale: 0.0,
        code: 0,
    },
    WoodsInfo {
        kind: Woods::Few,
        name: "Few",
        scale: 0.35,
        code: 1,
    },
    WoodsInfo {
        kind: Woods::Normal,
        name: "Normal",
        scale: 1.0,
        code: 2,
    },
    WoodsInfo {
        kind: Woods::Thick,
        name: "Thick",
        scale: 2.4,
        code: 3,
    },
];

impl Woods {
    pub fn name(self) -> &'static str {
        WOODS[self as usize].name
    }

    pub fn scale(self) -> f32 {
        WOODS[self as usize].scale
    }

    pub fn code(self) -> u8 {
        WOODS[self as usize].code
    }

    pub fn next(self) -> Woods {
        WOODS[(self as usize + 1) % WOODS.len()].kind
    }

    pub fn from_code(code: u8) -> Option<Woods> {
        WOODS.get(code as usize).map(|row| row.kind)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
#[serde(into = "u8", try_from = "u8")]
pub enum StartOfDay {
    #[default]
    Morning,
    Noon,
    Evening,
    Night,
}

#[derive(Clone, Copy, Debug)]
pub struct StartOfDayInfo {
    pub kind: StartOfDay,
    pub name: &'static str,
    pub hour: f32,
    pub code: u8,
}

pub const START_OF_DAY: [StartOfDayInfo; 4] = [
    StartOfDayInfo {
        kind: StartOfDay::Morning,
        name: "Morning",
        hour: theme::START_HOUR,
        code: 0,
    },
    StartOfDayInfo {
        kind: StartOfDay::Noon,
        name: "Noon",
        hour: 12.0,
        code: 1,
    },
    StartOfDayInfo {
        kind: StartOfDay::Evening,
        name: "Evening",
        hour: 17.5,
        code: 2,
    },
    StartOfDayInfo {
        kind: StartOfDay::Night,
        name: "Night",
        hour: 22.0,
        code: 3,
    },
];

impl StartOfDay {
    pub fn name(self) -> &'static str {
        START_OF_DAY[self as usize].name
    }

    pub fn hour(self) -> f32 {
        START_OF_DAY[self as usize].hour
    }

    pub fn code(self) -> u8 {
        START_OF_DAY[self as usize].code
    }

    pub fn next(self) -> StartOfDay {
        START_OF_DAY[(self as usize + 1) % START_OF_DAY.len()].kind
    }

    pub fn from_code(code: u8) -> Option<StartOfDay> {
        START_OF_DAY.get(code as usize).map(|row| row.kind)
    }
}

pub const CRACK_STAGES: [Cell; 8] = [
    (0, 7),
    (1, 7),
    (2, 7),
    (3, 7),
    (4, 7),
    (5, 7),
    (6, 7),
    (7, 7),
];

#[derive(Clone, Copy, Debug)]
pub struct ToolInfo {
    pub kind: ToolKind,
    pub name: &'static str,
    pub art: [&'static str; 16],
}

pub const TOOLS: [ToolInfo; 4] = [
    ToolInfo {
        kind: ToolKind::Pickaxe,
        name: "Pickaxe",
        art: [
            "................",
            "......iiiiiii...",
            ".....iIIIIIIIi..",
            "....iIdddddIIi..",
            "....id.....dIi..",
            "....d....bB.d...",
            "........bB......",
            ".......bB.......",
            "......bB........",
            ".....bB.........",
            "....bB..........",
            "...bB...........",
            "..bB............",
            ".bB.............",
            ".BB.............",
            "................",
        ],
    },
    ToolInfo {
        kind: ToolKind::Shovel,
        name: "Shovel",
        art: [
            "................",
            "................",
            ".......iiii.....",
            "......iIIIIi....",
            "......iIIIIi....",
            "......iIIIIi....",
            ".......iIIi.....",
            ".......bB.......",
            "......bB........",
            ".....bB.........",
            "....bB..........",
            "...bB...........",
            "..bB............",
            ".bB.............",
            ".BB.............",
            "................",
        ],
    },
    ToolInfo {
        kind: ToolKind::Axe,
        name: "Axe",
        art: [
            "................",
            "....iiiii.......",
            "...iIIIIi.......",
            "..iIIIIIIi......",
            "..iIIIIIIi......",
            "...iIIIIi.bB....",
            "....iIIi.bB.....",
            "........bB......",
            ".......bB.......",
            "......bB........",
            ".....bB.........",
            "....bB..........",
            "...bB...........",
            "..bB............",
            ".BB.............",
            "................",
        ],
    },
    ToolInfo {
        kind: ToolKind::Sword,
        name: "Sword",
        art: [
            "................",
            "............iii.",
            "...........iiIi.",
            "..........iiIi..",
            ".........iiIi...",
            "........iiIi....",
            ".......iiIi.....",
            "......iiIi......",
            ".....iiIi.......",
            "....iiIi........",
            "...biIi.........",
            "..bBb...........",
            ".bBBb...........",
            ".BBb............",
            ".Bb.............",
            "................",
        ],
    },
];

impl ToolKind {
    pub fn name(self) -> &'static str {
        TOOLS[self as usize].name
    }

    pub fn speed(self) -> f32 {
        6.0
    }

    pub fn art(self) -> [&'static str; 16] {
        TOOLS[self as usize].art
    }
}

pub const CHEST_ART: [((u32, u32), [&str; 16]); 2] = [
    (
        theme::CHEST_TOP_CELL,
        [
            "BBBBBBBBBBBBBBBB",
            "BbbbbbbbbbbbbbbB",
            "BbttttttttttttbB",
            "BbtbbbbbbbbbbtbB",
            "BbtbBBBBBBBBbtbB",
            "BbtbBbbbbbbBbtbB",
            "BbtbBbbbbbbBbtbB",
            "BbtbBbbbbbbBbtbB",
            "BbtbBbbbbbbBbtbB",
            "BbtbBbbbbbbBbtbB",
            "BbtbBbbbbbbBbtbB",
            "BbtbBBBBBBBBbtbB",
            "BbtbbbbbbbbbbtbB",
            "BbttttttttttttbB",
            "BbbbbbbbbbbbbbbB",
            "BBBBBBBBBBBBBBBB",
        ],
    ),
    (
        theme::CHEST_SIDE_CELL,
        [
            "BBBBBBBBBBBBBBBB",
            "BttttttttttttttB",
            "BtbbbbbbbbbbbbtB",
            "BtbbbbbbbbbbbbtB",
            "BtbbbbbbbbbbbbtB",
            "BBBBBBBYYBBBBBBB",
            "BtttttbyybbtttbB",
            "BtbbbbbyybbbbbtB",
            "BtbbbbbYYbbbbbtB",
            "BtbbbbbbbbbbbbtB",
            "BtbbbbbbbbbbbbtB",
            "BtbbbbbbbbbbbbtB",
            "BtbbbbbbbbbbbbtB",
            "BtbbbbbbbbbbbbtB",
            "BttttttttttttttB",
            "BBBBBBBBBBBBBBBB",
        ],
    ),
];

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

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, Serialize, Deserialize)]
#[serde(into = "u8", try_from = "u8")]
pub enum CropKind {
    #[default]
    Carrot,
    Wheat,
}

#[derive(Clone, Copy, Debug)]
pub struct CropInfo {
    pub kind: CropKind,
    pub name: &'static str,
    pub seed_name: &'static str,
    pub stages: [u32; 4],
    pub grow_seconds: f32,
}

pub const CROPS: [CropInfo; 2] = [
    CropInfo {
        kind: CropKind::Carrot,
        name: "Carrot",
        seed_name: "Carrot Seeds",
        stages: [5, 6, 7, 8],
        grow_seconds: 50.0,
    },
    CropInfo {
        kind: CropKind::Wheat,
        name: "Wheat",
        seed_name: "Wheat Seeds",
        stages: [64, 65, 66, 67],
        grow_seconds: 40.0,
    },
];

impl CropKind {
    pub fn info(self) -> CropInfo {
        CROPS[self as usize]
    }

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn from_index(index: u8) -> Option<CropKind> {
        CROPS.get(index as usize).map(|row| row.kind)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
#[serde(into = "[u8; 2]", try_from = "[u8; 2]")]
pub enum Item {
    Block(BlockKind),
    Tool(ToolKind),
    Firework,
    Seed(CropKind),
    Crop(CropKind),
    Bucket,
    WaterBucket,
    Loot(usize),
}

impl Item {
    pub fn code(self) -> [u8; 2] {
        match self {
            Item::Block(kind) => [0, kind.index()],
            Item::Tool(tool) => [1, tool.index()],
            Item::Firework => [2, 0],
            Item::Seed(crop) => [3, crop.index() as u8],
            Item::Crop(crop) => [4, crop.index() as u8],
            Item::Bucket => [5, 0],
            Item::WaterBucket => [6, 0],
            Item::Loot(kind) => [7, kind as u8],
        }
    }

    pub fn from_code(code: [u8; 2]) -> Option<Item> {
        match code[0] {
            0 => Some(Item::Block(BlockKind::from_index(code[1]))),
            1 => ToolKind::from_index(code[1]).map(Item::Tool),
            2 => Some(Item::Firework),
            3 => CropKind::from_index(code[1]).map(Item::Seed),
            4 => CropKind::from_index(code[1]).map(Item::Crop),
            5 => Some(Item::Bucket),
            6 => Some(Item::WaterBucket),
            7 => (usize::from(code[1]) < PICKUP_COUNT).then_some(Item::Loot(usize::from(code[1]))),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Item::Block(kind) => kind.info().name,
            Item::Tool(tool) => tool.name(),
            Item::Firework => "Firework",
            Item::Loot(kind) => PICKUPS[kind].name,
            Item::Bucket => "Bucket",
            Item::WaterBucket => "Water Bucket",
            Item::Seed(crop) => crop.info().seed_name,
            Item::Crop(crop) => crop.info().name,
        }
    }

    pub fn max_stack(self) -> u32 {
        match self {
            Item::Tool(_) => 1,
            _ => 64,
        }
    }

    pub fn art_index(self) -> Option<usize> {
        match self {
            Item::Tool(tool) => Some(match tool {
                ToolKind::Pickaxe => 0,
                ToolKind::Shovel => 1,
                ToolKind::Axe => 2,
                ToolKind::Sword => 3,
            }),
            Item::Firework => Some(4),
            Item::Seed(crop) => Some(5 + crop.index()),
            Item::Crop(crop) => Some(7 + crop.index()),
            Item::Bucket => Some(9),
            Item::WaterBucket => Some(10),
            Item::Loot(kind) => Some(11 + kind),
            Item::Block(_) => None,
        }
    }

    pub fn art(self) -> Option<[&'static str; 16]> {
        match self {
            Item::Tool(tool) => Some(tool.art()),
            Item::Firework => Some([
                "................",
                "..........rr....",
                ".........rwwr...",
                "........rwwwwr..",
                "........rwuuwr..",
                "........rwuuwr..",
                "........rwwwwr..",
                "........rrrrrr..",
                ".......bBrrrrBb.",
                "......bB..yy..B.",
                ".....bB...yy....",
                "....bB....yy....",
                "...bB....yooy...",
                "..bB.....oooo...",
                ".BB.......oo....",
                "................",
            ]),
            Item::Loot(0) => Some([
                "................",
                "......g.g.......",
                ".....gGgGg......",
                "....gGgggGg.....",
                ".....ggGgg......",
                "......tTt.......",
                ".....tTTTt......",
                ".....tTtTt......",
                "....tTtttTt.....",
                "....tTtttTt.....",
                ".....tTtTt......",
                "......ttt.......",
                ".......t........",
                "................",
                "................",
                "................",
            ]),
            Item::Loot(1) => Some([
                "................",
                "................",
                "................",
                "..........uu....",
                ".......uuuuuu...",
                "....uuuuuuuuuu..",
                "..uuuuuwuuuuuuu.",
                ".uuuuuuuuuuuuuuu",
                "..uuuuuuuuuuuuu.",
                "....uuuuuuuuuu..",
                ".......uuuuuu...",
                "..........uu....",
                "................",
                "................",
                "................",
                "................",
            ]),
            Item::Loot(_) => Some([
                "................",
                "................",
                "......dddd......",
                ".....ddIIdd.....",
                "....ddIooIdd....",
                "...ddIooooIdd...",
                "...dIooooooId...",
                "...dIoooooIdd...",
                "...ddIoooIddd...",
                "....ddIIIddd....",
                ".....dddddd.....",
                "......dddd......",
                "................",
                "................",
                "................",
                "................",
            ]),
            Item::Bucket => Some([
                "................",
                "................",
                "................",
                "..II........II..",
                "..II........II..",
                "..iIIIIIIIIIIi..",
                "..iIddddddddIi..",
                "..iIddddddddIi..",
                "...iIdddddddIi..",
                "...iIddddddIi...",
                "....iIdddddIi...",
                "....iIIIIIIIi...",
                ".....iiiiiii....",
                "................",
                "................",
                "................",
            ]),
            Item::WaterBucket => Some([
                "................",
                "................",
                "................",
                "..II........II..",
                "..II........II..",
                "..iIIIIIIIIIIi..",
                "..iIuuuuuuuuIi..",
                "..iIuuuuuuuuIi..",
                "...iIuuuuuuuIi..",
                "...iIuuuuuuIi...",
                "....iIuuuuuIi...",
                "....iIIIIIIIi...",
                ".....iiiiiii....",
                "................",
                "................",
                "................",
            ]),
            Item::Seed(CropKind::Carrot) => Some([
                "................",
                "................",
                ".....tttttt.....",
                "....tTttttTt....",
                "...tTttttttTt...",
                "...tttooootTt...",
                "...ttoooooott...",
                "...ttoooooott...",
                "...tttooootTt...",
                "...tTttttttTt...",
                "....tTttttTt....",
                ".....tttttt.....",
                "................",
                "................",
                "................",
                "................",
            ]),
            Item::Seed(CropKind::Wheat) => Some([
                "................",
                "................",
                ".....tttttt.....",
                "....tTttttTt....",
                "...tTttttttTt...",
                "...tttyyyytTt...",
                "...ttyyyyyytt...",
                "...ttyyyyyytt...",
                "...tttyyyytTt...",
                "...tTttttttTt...",
                "....tTttttTt....",
                ".....tttttt.....",
                "................",
                "................",
                "................",
                "................",
            ]),
            Item::Crop(CropKind::Carrot) => Some([
                "................",
                "..........gg....",
                ".........gGg....",
                "........gGgg....",
                ".......gGg......",
                "......ooGg......",
                ".....oooo.......",
                "....ooooo.......",
                "....oooo........",
                "...oooo.........",
                "...ooo..........",
                "..ooo...........",
                "..oo............",
                ".oo.............",
                "................",
                "................",
            ]),
            Item::Crop(CropKind::Wheat) => Some([
                "................",
                "..........yy....",
                ".........yYy....",
                "........yYyy....",
                ".......yYy......",
                "......yYyy......",
                ".....yYy........",
                "....yYyy........",
                "...yYyY.........",
                "...tYt..........",
                "..tT............",
                "..tT............",
                ".tT.............",
                ".tT.............",
                "................",
                "................",
            ]),
            Item::Block(_) => None,
        }
    }

    pub fn mesh_name(self) -> String {
        match self {
            Item::Block(kind) => format!("voxels_item_block_{}", kind.index()),
            _ => format!("voxels_item_art_{}", self.art_index().unwrap_or(0)),
        }
    }

    pub fn material_name(self) -> &'static str {
        match self {
            Item::Block(kind) if kind.info().transparent => theme::TRANSPARENT_MATERIAL,
            Item::Block(kind) if kind.info().cutout => theme::CUTOUT_MATERIAL,
            Item::Block(_) => theme::SOLID_MATERIAL,
            _ => theme::FLAT_MATERIAL,
        }
    }
}

pub const ART_ITEMS: [Item; 14] = [
    Item::Tool(ToolKind::Pickaxe),
    Item::Tool(ToolKind::Shovel),
    Item::Tool(ToolKind::Axe),
    Item::Tool(ToolKind::Sword),
    Item::Firework,
    Item::Seed(CropKind::Carrot),
    Item::Seed(CropKind::Wheat),
    Item::Crop(CropKind::Carrot),
    Item::Crop(CropKind::Wheat),
    Item::Bucket,
    Item::WaterBucket,
    Item::Loot(0),
    Item::Loot(1),
    Item::Loot(2),
];

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct ItemStack {
    pub item: Item,
    pub count: u32,
}

pub fn starting_inventory() -> [Option<ItemStack>; theme::INVENTORY_SLOTS] {
    let stack = |item, count| Some(ItemStack { item, count });
    let given = [
        stack(Item::Tool(ToolKind::Pickaxe), 1),
        stack(Item::Tool(ToolKind::Shovel), 1),
        stack(Item::Tool(ToolKind::Axe), 1),
        stack(Item::Tool(ToolKind::Sword), 1),
        stack(Item::Block(BlockKind::OakPlanks), 32),
        stack(Item::Seed(CropKind::Carrot), 4),
        stack(Item::WaterBucket, 1),
        stack(Item::Block(BlockKind::Torch), 16),
        stack(Item::Block(BlockKind::Glass), 16),
    ];
    let mut slots = [None; theme::INVENTORY_SLOTS];
    slots[..given.len()].copy_from_slice(&given);
    slots
}

#[derive(Clone, Copy, Debug)]
pub struct Recipe {
    pub output: Item,
    pub count: u32,
    pub inputs: &'static [(Item, u32)],
}

pub const RECIPES: [Recipe; 16] = [
    Recipe {
        output: Item::Block(BlockKind::Chest),
        count: 1,
        inputs: &[(Item::Block(BlockKind::OakPlanks), 8)],
    },
    Recipe {
        output: Item::Bucket,
        count: 1,
        inputs: &[(Item::Block(BlockKind::IronOre), 3)],
    },
    Recipe {
        output: Item::Block(BlockKind::DoorNorth),
        count: 1,
        inputs: &[(Item::Block(BlockKind::OakPlanks), 6)],
    },
    Recipe {
        output: Item::Block(BlockKind::Torch),
        count: 4,
        inputs: &[
            (Item::Block(BlockKind::OakPlanks), 1),
            (Item::Block(BlockKind::CoalOre), 1),
        ],
    },
    Recipe {
        output: Item::Block(BlockKind::OakPlanks),
        count: 4,
        inputs: &[(Item::Block(BlockKind::OakLog), 1)],
    },
    Recipe {
        output: Item::Block(BlockKind::CraftingTable),
        count: 1,
        inputs: &[(Item::Block(BlockKind::OakPlanks), 4)],
    },
    Recipe {
        output: Item::Block(BlockKind::Signpost),
        count: 1,
        inputs: &[
            (Item::Block(BlockKind::OakPlanks), 2),
            (Item::Block(BlockKind::OakLog), 1),
        ],
    },
    Recipe {
        output: Item::Block(BlockKind::Campfire),
        count: 1,
        inputs: &[
            (Item::Block(BlockKind::Cobblestone), 4),
            (Item::Block(BlockKind::OakLog), 2),
        ],
    },
    Recipe {
        output: Item::Block(BlockKind::Glass),
        count: 1,
        inputs: &[(Item::Block(BlockKind::Sand), 2)],
    },
    Recipe {
        output: Item::Block(BlockKind::Bricks),
        count: 1,
        inputs: &[(Item::Block(BlockKind::Cobblestone), 4)],
    },
    Recipe {
        output: Item::Firework,
        count: 2,
        inputs: &[
            (Item::Block(BlockKind::Gravel), 1),
            (Item::Block(BlockKind::Glowstone), 1),
            (Item::Block(BlockKind::OakPlanks), 1),
        ],
    },
    Recipe {
        output: Item::Block(BlockKind::WoolRed),
        count: 1,
        inputs: &[
            (Item::Block(BlockKind::WoolWhite), 1),
            (Item::Block(BlockKind::RedFlower), 1),
        ],
    },
    Recipe {
        output: Item::Tool(ToolKind::Pickaxe),
        count: 1,
        inputs: &[
            (Item::Block(BlockKind::Cobblestone), 3),
            (Item::Block(BlockKind::OakPlanks), 2),
        ],
    },
    Recipe {
        output: Item::Tool(ToolKind::Shovel),
        count: 1,
        inputs: &[
            (Item::Block(BlockKind::Cobblestone), 1),
            (Item::Block(BlockKind::OakPlanks), 2),
        ],
    },
    Recipe {
        output: Item::Tool(ToolKind::Axe),
        count: 1,
        inputs: &[
            (Item::Block(BlockKind::Cobblestone), 3),
            (Item::Block(BlockKind::OakPlanks), 2),
        ],
    },
    Recipe {
        output: Item::Seed(CropKind::Wheat),
        count: 2,
        inputs: &[(Item::Crop(CropKind::Wheat), 1)],
    },
];

#[derive(Clone, Copy, Debug)]
pub struct ShopRow {
    pub item: Item,
    pub buy: Option<u32>,
    pub sell: Option<u32>,
}

pub const SHOP: [ShopRow; 16] = [
    ShopRow {
        item: Item::Block(BlockKind::Torch),
        buy: Some(40),
        sell: None,
    },
    ShopRow {
        item: Item::Firework,
        buy: Some(180),
        sell: None,
    },
    ShopRow {
        item: Item::Seed(CropKind::Carrot),
        buy: Some(90),
        sell: None,
    },
    ShopRow {
        item: Item::Seed(CropKind::Wheat),
        buy: Some(50),
        sell: None,
    },
    ShopRow {
        item: Item::Block(BlockKind::Glowstone),
        buy: Some(300),
        sell: None,
    },
    ShopRow {
        item: Item::Block(BlockKind::Tnt),
        buy: Some(400),
        sell: None,
    },
    ShopRow {
        item: Item::Crop(CropKind::Carrot),
        buy: None,
        sell: Some(120),
    },
    ShopRow {
        item: Item::Crop(CropKind::Wheat),
        buy: None,
        sell: Some(80),
    },
    ShopRow {
        item: Item::Block(BlockKind::OakLog),
        buy: None,
        sell: Some(35),
    },
    ShopRow {
        item: Item::Block(BlockKind::CoalOre),
        buy: None,
        sell: Some(90),
    },
    ShopRow {
        item: Item::Block(BlockKind::IronOre),
        buy: None,
        sell: Some(180),
    },
    ShopRow {
        item: Item::Block(BlockKind::GoldOre),
        buy: None,
        sell: Some(400),
    },
    ShopRow {
        item: Item::Block(BlockKind::DiamondOre),
        buy: None,
        sell: Some(1000),
    },
    ShopRow {
        item: Item::Loot(0),
        buy: None,
        sell: Some(theme::LOOT_PRICE[0]),
    },
    ShopRow {
        item: Item::Loot(1),
        buy: None,
        sell: Some(theme::LOOT_PRICE[1]),
    },
    ShopRow {
        item: Item::Loot(2),
        buy: None,
        sell: Some(theme::LOOT_PRICE[2]),
    },
];

pub const SIGN_TEXT: &str = "Sign";

#[derive(Clone, Copy, Debug)]
pub struct BiomeInfo {
    pub base_height: f32,
    pub relief: f32,
    pub top: BlockKind,
    pub filler: BlockKind,
    pub trees_per_hundred: f32,
    pub flowers_per_hundred: f32,
    pub bushes_per_hundred: f32,
    pub pines: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum WeatherKind {
    #[default]
    Clear,
    Cloudy,
    Rain,
    Storm,
    Snowfall,
}

impl WeatherKind {
    pub fn wets(self) -> bool {
        matches!(self, WeatherKind::Rain | WeatherKind::Storm)
    }

    pub fn snows(self) -> bool {
        self == WeatherKind::Snowfall
    }

    pub fn clouds(self) -> f32 {
        match self {
            WeatherKind::Clear => 0.0,
            WeatherKind::Cloudy => 0.55,
            WeatherKind::Rain => 0.85,
            WeatherKind::Storm => 1.0,
            WeatherKind::Snowfall => 0.75,
        }
    }

    pub fn word(self) -> &'static str {
        match self {
            WeatherKind::Clear => "The sky clears",
            WeatherKind::Cloudy => "Clouds roll in",
            WeatherKind::Rain => "Rain starts to fall",
            WeatherKind::Storm => "A storm breaks",
            WeatherKind::Snowfall => "Snow starts to fall",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Biome {
    Plains,
    Forest,
    Desert,
    Hills,
    Ocean,
    Snowfield,
    Peaks,
    Swamp,
    Badlands,
}

pub const BIOMES: usize = 9;

impl Biome {
    pub const ALL: [Biome; BIOMES] = [
        Biome::Plains,
        Biome::Forest,
        Biome::Desert,
        Biome::Hills,
        Biome::Ocean,
        Biome::Snowfield,
        Biome::Peaks,
        Biome::Swamp,
        Biome::Badlands,
    ];

    pub fn is_cold(self) -> bool {
        matches!(self, Biome::Snowfield | Biome::Peaks)
    }

    pub fn is_parched(self) -> bool {
        matches!(self, Biome::Desert | Biome::Badlands)
    }

    pub fn rockiness(self) -> f32 {
        match self {
            Biome::Peaks => 1.0,
            Biome::Badlands => 0.85,
            Biome::Hills => 0.7,
            Biome::Snowfield => 0.45,
            Biome::Desert => 0.3,
            Biome::Forest => 0.26,
            Biome::Plains => 0.22,
            Biome::Ocean => 0.18,
            Biome::Swamp => 0.1,
        }
    }

    pub fn info(self) -> BiomeInfo {
        match self {
            Biome::Plains => BiomeInfo {
                base_height: 26.0,
                relief: 3.0,
                top: BlockKind::Grass,
                filler: BlockKind::Dirt,
                trees_per_hundred: 0.4,
                flowers_per_hundred: 3.0,
                bushes_per_hundred: 0.2,
                pines: false,
            },
            Biome::Forest => BiomeInfo {
                base_height: 28.0,
                relief: 5.0,
                top: BlockKind::Grass,
                filler: BlockKind::Dirt,
                trees_per_hundred: 5.0,
                flowers_per_hundred: 1.0,
                bushes_per_hundred: 0.2,
                pines: false,
            },
            Biome::Desert => BiomeInfo {
                base_height: 25.0,
                relief: 2.5,
                top: BlockKind::Sand,
                filler: BlockKind::Sand,
                trees_per_hundred: 0.0,
                flowers_per_hundred: 0.0,
                bushes_per_hundred: 1.0,
                pines: false,
            },
            Biome::Hills => BiomeInfo {
                base_height: 34.0,
                relief: 14.0,
                top: BlockKind::Grass,
                filler: BlockKind::Dirt,
                trees_per_hundred: 1.0,
                flowers_per_hundred: 0.5,
                bushes_per_hundred: 0.1,
                pines: false,
            },
            Biome::Ocean => BiomeInfo {
                base_height: 12.0,
                relief: 2.5,
                top: BlockKind::Sand,
                filler: BlockKind::Gravel,
                trees_per_hundred: 0.0,
                flowers_per_hundred: 0.0,
                bushes_per_hundred: 0.0,
                pines: false,
            },
            Biome::Snowfield => BiomeInfo {
                base_height: 27.0,
                relief: 4.0,
                top: BlockKind::SnowGrass,
                filler: BlockKind::Dirt,
                trees_per_hundred: 2.0,
                flowers_per_hundred: 0.0,
                bushes_per_hundred: 0.4,
                pines: true,
            },
            Biome::Peaks => BiomeInfo {
                base_height: 42.0,
                relief: 18.0,
                top: BlockKind::Snow,
                filler: BlockKind::Stone,
                trees_per_hundred: 0.2,
                flowers_per_hundred: 0.0,
                bushes_per_hundred: 0.0,
                pines: true,
            },
            Biome::Swamp => BiomeInfo {
                base_height: 23.0,
                relief: 1.5,
                top: BlockKind::Grass,
                filler: BlockKind::Dirt,
                trees_per_hundred: 2.5,
                flowers_per_hundred: 0.4,
                bushes_per_hundred: 1.6,
                pines: false,
            },
            Biome::Badlands => BiomeInfo {
                base_height: 31.0,
                relief: 11.0,
                top: BlockKind::RedSand,
                filler: BlockKind::Terracotta,
                trees_per_hundred: 0.0,
                flowers_per_hundred: 0.0,
                bushes_per_hundred: 1.2,
                pines: false,
            },
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Building {
    pub sink: i32,
    pub layers: &'static [&'static [&'static str]],
}

impl Building {
    pub fn width(&self) -> i32 {
        self.layers
            .iter()
            .flat_map(|layer| layer.iter())
            .map(|row| row.len() as i32)
            .max()
            .unwrap_or(0)
    }

    pub fn depth(&self) -> i32 {
        self.layers
            .iter()
            .map(|layer| layer.len() as i32)
            .max()
            .unwrap_or(0)
    }

    pub fn doorway(&self) -> Option<i32> {
        let ground = self.layers.get(1)?;
        if let Some(column) = ground.last().and_then(|row| row.find('D')) {
            return Some(column as i32);
        }
        let column = self.width() / 2;
        let open = |layer: &&[&str]| {
            layer
                .last()
                .and_then(|row| row.chars().nth(column as usize))
                == Some('.')
        };
        (open(ground) && self.layers.get(2).is_some_and(open)).then_some(column)
    }
}

pub fn building_block(character: char) -> Option<BlockKind> {
    match character {
        'C' => Some(BlockKind::Cobblestone),
        'P' => Some(BlockKind::OakPlanks),
        'L' => Some(BlockKind::OakLog),
        'G' => Some(BlockKind::Glass),
        'B' => Some(BlockKind::Bricks),
        'T' => Some(BlockKind::CraftingTable),
        'W' => Some(BlockKind::WoolWhite),
        'R' => Some(BlockKind::WoolRed),
        'U' => Some(BlockKind::Water),
        'g' => Some(BlockKind::Glowstone),
        'F' => Some(BlockKind::Campfire),
        'D' => Some(BlockKind::DoorSouth),
        'S' => Some(BlockKind::Stone),
        'V' => Some(BlockKind::Gravel),
        'l' => Some(BlockKind::OakLeaves),
        'Y' => Some(BlockKind::YellowFlower),
        'r' => Some(BlockKind::RedFlower),
        't' => Some(BlockKind::Torch),
        'h' => Some(BlockKind::Chest),
        'd' => Some(BlockKind::Dirt),
        'b' => Some(BlockKind::WoolBlue),
        '.' => Some(BlockKind::Air),
        _ => None,
    }
}

const SMALL_HOME: Building = Building {
    sink: 0,
    layers: &[
        &[
            "CCCCCCC", "CPPPPPC", "CPPPPPC", "CPPPPPC", "CPPPPPC", "CCCCCCC",
        ],
        &[
            "LPGPGPL", "P.....P", "G.....G", "P.....P", "G.T...G", "LPGDGPL",
        ],
        &[
            "LPGGGPL", "P.....P", "G.....G", "P.....P", "G.....G", "LPGDGPL",
        ],
        &[
            "LPPPPPL", "P..g..P", "P.....P", "P.....P", "P.....P", "LPPPPPL",
        ],
        &[
            "CCCCCCC", "CPPPPPC", "CPPPPPC", "CPPPPPC", "CPPPPPC", "CCCCCCC",
        ],
        &[
            "       ", " CCCCC ", " C...C ", " C...C ", " CCCCC ", "       ",
        ],
    ],
};

const LONG_HOME: Building = Building {
    sink: 0,
    layers: &[
        &[
            "CCCCCCCCC",
            "CPPPPPPPC",
            "CPPPPPPPC",
            "CPPPPPPPC",
            "CPPPPPPPC",
            "CPPPPPPPC",
            "CCCCCCCCC",
        ],
        &[
            "LPGPGPGPL",
            "P.......P",
            "G.......G",
            "P.......P",
            "G.......G",
            "P.T.....P",
            "LPGPDPGPL",
        ],
        &[
            "LPGPGPGPL",
            "P.......P",
            "G.......G",
            "P.......P",
            "G.......G",
            "P.......P",
            "LPGPDPGPL",
        ],
        &[
            "LPPPPPPPL",
            "P...g...P",
            "P.......P",
            "P.......P",
            "P.......P",
            "P.......P",
            "LPPPPPPPL",
        ],
        &[
            "BBBBBBBBB",
            "BBBBBBBBB",
            "BBBBBBBBB",
            "BBBBBBBBB",
            "BBBBBBBBB",
            "BBBBBBBBB",
            "BBBBBBBBB",
        ],
        &[
            "         ",
            " BBBBBBB ",
            " B.....B ",
            " B.....B ",
            " B.....B ",
            " BBBBBBB ",
            "         ",
        ],
        &[
            "         ",
            "         ",
            "  BBBBB  ",
            "  BBBBB  ",
            "  BBBBB  ",
            "         ",
            "         ",
        ],
    ],
};

const SHOP_HOUSE: Building = Building {
    sink: 0,
    layers: &[
        &[
            "CCCCCCCC", "CPPPPPPC", "CPPPPPPC", "CPPPPPPC", "CPPPPPPC", "CCCCCCCC",
        ],
        &[
            "LGGGGGGL", "G......G", "P......P", "PTTT...P", "G......G", "LPGPDPGL",
        ],
        &[
            "LGGGGGGL", "G......G", "P......P", "P......P", "G......G", "LPGPDPGL",
        ],
        &[
            "LPPPPPPL", "P.g..g.P", "P......P", "P......P", "P......P", "LPPPPPPL",
        ],
        &[
            "RWRWRWRW", "WPPPPPPW", "RPPPPPPR", "WPPPPPPW", "RPPPPPPR", "WRWRWRWR",
        ],
        &[
            "RWRWRWRW", "        ", "        ", "        ", "        ", "        ",
        ],
    ],
};

const WELL: Building = Building {
    sink: 0,
    layers: &[
        &["CCCC", "CUUC", "CUUC", "CCCC"],
        &["C..C", "....", "....", "C..C"],
        &["L..L", "....", "....", "L..L"],
        &["L..L", "....", "....", "L..L"],
        &["PPPP", "PPPP", "PPPP", "PPPP"],
    ],
};

const FIRE_PIT: Building = Building {
    sink: 0,
    layers: &[&["CCC", "CFC", "CCC"]],
};

const KEEP: Building = Building {
    sink: 0,
    layers: &[
        &[
            "CCCCCCCCCCC",
            "CCCCCCCCCCC",
            "CCCCCCCCCCC",
            "CCCCCCCCCCC",
            "CCCCCCCCCCC",
            "CCCCCCCCCCC",
            "CCCCCCCCCCC",
            "CCCCCCCCCCC",
            "CCCCCCCCCCC",
        ],
        &[
            "CCCCCCCCCCC",
            "C.........C",
            "C.........C",
            "C.........C",
            "C....g....C",
            "C.........C",
            "C.........C",
            "C.........C",
            "CCCCCDCCCCC",
        ],
        &[
            "CCGCCCCCGCC",
            "C.........C",
            "C.........C",
            "G.........G",
            "C.........C",
            "G.........G",
            "C.........C",
            "C.........C",
            "CCGCCDCCGCC",
        ],
        &[
            "CCCCCCCCCCC",
            "C.........C",
            "C.........C",
            "C.........C",
            "C.........C",
            "C.........C",
            "C.........C",
            "C.........C",
            "CCCCCCCCCCC",
        ],
        &[
            "CCCCCCCCCCC",
            "CPPPPPPPPPC",
            "CPPPPPPPPPC",
            "CPPPPPPPPPC",
            "CPPPPgPPPPC",
            "CPPPPPPPPPC",
            "CPPPPPPPPPC",
            "CPPPPPPPPPC",
            "CCCCCCCCCCC",
        ],
        &[
            "CCGCCCCCGCC",
            "C.........C",
            "G.........G",
            "C.........C",
            "C.........C",
            "C.........C",
            "G.........G",
            "C.........C",
            "CCGCCCCCGCC",
        ],
        &[
            "CCCCCCCCCCC",
            "C.........C",
            "C.........C",
            "C.........C",
            "C.........C",
            "C.........C",
            "C.........C",
            "C.........C",
            "CCCCCCCCCCC",
        ],
        &[
            "CCCCCCCCCCC",
            "CCCCCCCCCCC",
            "CCCCCCCCCCC",
            "CCCCCCCCCCC",
            "CCCCCCCCCCC",
            "CCCCCCCCCCC",
            "CCCCCCCCCCC",
            "CCCCCCCCCCC",
            "CCCCCCCCCCC",
        ],
        &[
            "C.C.C.C.C.C",
            ".........R.",
            "C.........C",
            "...........",
            "C.........C",
            "...........",
            "C.........C",
            ".R.........",
            "C.C.C.C.C.C",
        ],
    ],
};

const CHAPEL: Building = Building {
    sink: 0,
    layers: &[
        &[
            "CCCCCCC", "CCCCCCC", "CCCCCCC", "CCCCCCC", "CCCCCCC", "CCCCCCC", "CCCCCCC", "CCCCCCC",
            "CCCCCCC", "CCCCCCC", "CCCCCCC",
        ],
        &[
            "CCCCCCC", "C.SgS.C", "G.....G", "C.....C", "G.....G", "C.....C", "G.....G", "C.....C",
            "G.....G", "C.....C", "CCGDGCC",
        ],
        &[
            "CCGGGCC", "C.....C", "G.....G", "C.....C", "G.....G", "C.....C", "G.....G", "C.....C",
            "G.....G", "C.....C", "CCGDGCC",
        ],
        &[
            "CCCCCCC", "C.....C", "C.....C", "C.....C", "C.....C", "C..g..C", "C.....C", "C.....C",
            "C.....C", "C.....C", "CCCCCCC",
        ],
        &[
            "BBBBBBB", "BBBBBBB", "BBBBBBB", "BBBBBBB", "BBBBBBB", "BBBBBBB", "BBBBBBB", "BBBBBBB",
            "BBBBBBB", "BBBBBBB", "BBBBBBB",
        ],
        &[
            " BBBBB ", " BBBBB ", " BBBBB ", " BBBBB ", " BBBBB ", " BBBBB ", " BBBBB ", " BBBBB ",
            " BBBBB ", " BBBBB ", " BBBBB ",
        ],
        &[
            "  BBB  ", "  BBB  ", "  BBB  ", "  BBB  ", "  BBB  ", "  BBB  ", "  BBB  ", "  BBB  ",
            "  BBB  ", "  CCC  ", "  CCC  ",
        ],
        &[
            "       ", "       ", "       ", "       ", "       ", "       ", "       ", "       ",
            "       ", "  C.C  ", "  C.C  ",
        ],
        &[
            "       ", "       ", "       ", "       ", "       ", "       ", "       ", "       ",
            "       ", "  t.t  ", "  t.t  ",
        ],
    ],
};

const SMITHY: Building = Building {
    sink: 0,
    layers: &[
        &[
            "CCCCCCC", "CCCCCCC", "CCCCCCC", "CCCCCCC", "CCCCCCC", "CCCCCCC", "CCCCCCC",
        ],
        &[
            "CCCCCCC", "C.....C", "C..F..C", "C.....C", "C.h...C", "C....SC", "L.....L",
        ],
        &[
            "CCCCCCC", "C.....C", "C.....C", "C.....C", "C.....C", "C.....C", "L.....L",
        ],
        &[
            "PPPPPPP", "PPCCCPP", "PPCCCPP", "PPCCCPP", "PPPPPPP", "PPPPPPP", "PPPPPPP",
        ],
        &[
            "       ", "  C.C  ", "  C.C  ", "  CCC  ", "       ", "       ", "       ",
        ],
        &[
            "       ", "  CCC  ", "  C.C  ", "  CCC  ", "       ", "       ", "       ",
        ],
    ],
};

const STALL: Building = Building {
    sink: 0,
    layers: &[
        &["L...L", ".....", ".PPP.", "L...L"],
        &["L...L", "..h..", ".....", "L...L"],
        &["L...L", ".....", ".....", "L...L"],
        &["RWRWR", "WRWRW", "RWRWR", "WRWRW"],
    ],
};

const BARN: Building = Building {
    sink: 0,
    layers: &[
        &[
            "CCCCCCCCC",
            "CCCCCCCCC",
            "CCCCCCCCC",
            "CCCCCCCCC",
            "CCCCCCCCC",
            "CCCCCCCCC",
            "CCCCCCCCC",
        ],
        &[
            "LPPPPPPPL",
            "P.......P",
            "P.L...L.P",
            "P.......P",
            "P.L...L.P",
            "P...h...P",
            "LPPP...PL",
        ],
        &[
            "LPPPPPPPL",
            "P.......P",
            "P.......P",
            "P.......P",
            "P.......P",
            "P.......P",
            "LPPP...PL",
        ],
        &[
            "LPPPPPPPL",
            "P.......P",
            "P...g...P",
            "P.......P",
            "P.......P",
            "P.......P",
            "LPPPPPPPL",
        ],
        &[
            "PPPPPPPPP",
            "PPPPPPPPP",
            "PPPPPPPPP",
            "PPPPPPPPP",
            "PPPPPPPPP",
            "PPPPPPPPP",
            "PPPPPPPPP",
        ],
        &[
            " PPPPPPP ",
            " PPPPPPP ",
            " PPPPPPP ",
            " PPPPPPP ",
            " PPPPPPP ",
            " PPPPPPP ",
            " PPPPPPP ",
        ],
        &[
            "  PPPPP  ",
            "  PPPPP  ",
            "  PPPPP  ",
            "  PPPPP  ",
            "  PPPPP  ",
            "  PPPPP  ",
            "  PPPPP  ",
        ],
    ],
};

const WATCHTOWER: Building = Building {
    sink: 0,
    layers: &[
        &["CCCCC", "CCCCC", "CCCCC", "CCCCC", "CCCCC"],
        &["CCCCC", "C...C", "C...C", "C...C", "CC.CC"],
        &["CCCCC", "C...C", "C...C", "C...C", "CC.CC"],
        &["CCCCC", "C...C", "C...C", "C...C", "CCCCC"],
        &["CCCCC", "C...C", "C...C", "C...C", "CCCCC"],
        &["CCGCC", "C...C", "G...G", "C...C", "CCGCC"],
        &["CCCCC", "C...C", "C...C", "C...C", "CCCCC"],
        &["CCCCC", "C.g.C", "C...C", "C...C", "CCCCC"],
        &["CCCCC", "CCCCC", "CCCCC", "CCCCC", "CCCCC"],
        &["C.C.C", ".....", "C...C", ".....", "C.C.C"],
        &["t...t", ".....", ".....", ".....", "t...t"],
    ],
};

const STABLE: Building = Building {
    sink: 0,
    layers: &[
        &["VVVVVVV", "VVVVVVV", "VVVVVVV", "VVVVVVV", "VVVVVVV"],
        &["L.L.L.L", ".......", "..P.P..", ".......", "L.L.L.L"],
        &["L.....L", ".......", ".......", ".......", "L.....L"],
        &["PPPPPPP", "PPPPPPP", "PPPPPPP", "PPPPPPP", "PPPPPPP"],
    ],
};

const GARDEN: Building = Building {
    sink: 0,
    layers: &[&["lllll", "lYrYl", "lrYrl", "lYrYl", "lllll"]],
};

const FOUNTAIN: Building = Building {
    sink: 0,
    layers: &[
        &["CCCCC", "CUUUC", "CUUUC", "CUUUC", "CCCCC"],
        &[".....", ".....", "..S..", ".....", "....."],
        &[".....", ".....", "..g..", ".....", "....."],
    ],
};

const LAMP: Building = Building {
    sink: 0,
    layers: &[&["L"], &["L"], &["t"]],
};

const BENCH: Building = Building {
    sink: 0,
    layers: &[&["PPP"]],
};

const FISH_HUT: Building = Building {
    sink: 0,
    layers: &[
        &["PPPPP", "PPPPP", "PPPPP", "PPPPP", "PPPPP"],
        &["LPGPL", "P...P", "P.h.P", "P...P", "LPDPL"],
        &["LPPPL", "P...P", "G...G", "P...P", "LPDPL"],
        &["PPPPP", "PPPPP", "PPPPP", "PPPPP", "PPPPP"],
        &[" PPP ", " PPP ", " PPP ", " PPP ", " PPP "],
    ],
};

const RACK: Building = Building {
    sink: 0,
    layers: &[&["L...L"], &["L...L"], &["PPPPP"]],
};

const FIELD_PLOT: Building = Building {
    sink: 1,
    layers: &[
        &[
            "ddddddddd",
            "UUUUUUUUU",
            "ddddddddd",
            "ddddddddd",
            "UUUUUUUUU",
            "ddddddddd",
            "ddddddddd",
        ],
        &[
            "LPLPLPLPL",
            "P.......P",
            "L.......L",
            "P.......P",
            "L.......L",
            "P.......P",
            "LPLPLPLPL",
        ],
    ],
};

const ALTAR: Building = Building {
    sink: 0,
    layers: &[
        &["S.S.S", ".....", "..C..", ".....", "S.S.S"],
        &["t...t", ".....", "..g..", ".....", "t...t"],
    ],
};

const CART: Building = Building {
    sink: 0,
    layers: &[&["L.L", "PPP", "PPP", "L.L"], &["...", "P.P", "PhP", "..."]],
};

pub const BUILDINGS: [Building; 21] = [
    WELL, SHOP_HOUSE, SMALL_HOME, FIRE_PIT, LONG_HOME, KEEP, CHAPEL, SMITHY, STALL, BARN,
    WATCHTOWER, STABLE, GARDEN, FOUNTAIN, LAMP, BENCH, FISH_HUT, RACK, FIELD_PLOT, ALTAR, CART,
];

pub const WELL_BUILDING: usize = 0;
pub const SHOP_BUILDING: usize = 1;
pub const HOME_BUILDING: usize = 2;
pub const FIRE_BUILDING: usize = 3;
pub const INN_BUILDING: usize = 4;
pub const KEEP_BUILDING: usize = 5;
pub const CHAPEL_BUILDING: usize = 6;
pub const SMITHY_BUILDING: usize = 7;
pub const STALL_BUILDING: usize = 8;
pub const BARN_BUILDING: usize = 9;
pub const TOWER_BUILDING: usize = 10;
pub const STABLE_BUILDING: usize = 11;
pub const GARDEN_BUILDING: usize = 12;
pub const FOUNTAIN_BUILDING: usize = 13;
pub const LAMP_BUILDING: usize = 14;
pub const BENCH_BUILDING: usize = 15;
pub const FISH_HUT_BUILDING: usize = 16;
pub const RACK_BUILDING: usize = 17;
pub const FIELD_BUILDING: usize = 18;
pub const ALTAR_BUILDING: usize = 19;
pub const CART_BUILDING: usize = 20;

pub const TOWN_PLAN: [usize; 20] = [
    FOUNTAIN_BUILDING,
    KEEP_BUILDING,
    CHAPEL_BUILDING,
    INN_BUILDING,
    SHOP_BUILDING,
    SMITHY_BUILDING,
    STALL_BUILDING,
    STALL_BUILDING,
    HOME_BUILDING,
    HOME_BUILDING,
    HOME_BUILDING,
    WELL_BUILDING,
    GARDEN_BUILDING,
    LAMP_BUILDING,
    LAMP_BUILDING,
    LAMP_BUILDING,
    LAMP_BUILDING,
    BENCH_BUILDING,
    BENCH_BUILDING,
    TOWER_BUILDING,
];
pub const FARM_PLAN: [usize; 10] = [
    FIRE_BUILDING,
    BARN_BUILDING,
    HOME_BUILDING,
    HOME_BUILDING,
    WELL_BUILDING,
    STABLE_BUILDING,
    FIELD_BUILDING,
    FIELD_BUILDING,
    LAMP_BUILDING,
    BENCH_BUILDING,
];
pub const FISHING_PLAN: [usize; 8] = [
    FIRE_BUILDING,
    FISH_HUT_BUILDING,
    FISH_HUT_BUILDING,
    FISH_HUT_BUILDING,
    RACK_BUILDING,
    RACK_BUILDING,
    WELL_BUILDING,
    LAMP_BUILDING,
];
pub const MINING_PLAN: [usize; 8] = [
    FIRE_BUILDING,
    SMITHY_BUILDING,
    HOME_BUILDING,
    HOME_BUILDING,
    CART_BUILDING,
    CART_BUILDING,
    TOWER_BUILDING,
    LAMP_BUILDING,
];
pub const SHRINE_PLAN: [usize; 6] = [
    ALTAR_BUILDING,
    GARDEN_BUILDING,
    GARDEN_BUILDING,
    LAMP_BUILDING,
    LAMP_BUILDING,
    HOME_BUILDING,
];
pub const HAMLET_PLAN: [usize; 6] = [
    FIRE_BUILDING,
    HOME_BUILDING,
    HOME_BUILDING,
    WELL_BUILDING,
    GARDEN_BUILDING,
    LAMP_BUILDING,
];

pub const VILLAGE_STARTS: [&str; 18] = [
    "Ash", "Elm", "Oak", "Thorn", "Wyn", "Bram", "Fen", "Mill", "Stone", "Ald", "Wolf", "Bright",
    "Cold", "Harrow", "Roan", "Birch", "Kest", "Marl",
];
pub const VILLAGE_ENDS: [&str; 14] = [
    "ford", "wick", "ton", "mere", "dale", "bury", "stead", "haven", "brook", "moor", "field",
    "gate", "well", "cross",
];

pub fn building_at(index: usize) -> &'static Building {
    BUILDINGS.get(index).unwrap_or(&WELL)
}

impl From<Land> for u8 {
    fn from(value: Land) -> Self {
        value.code()
    }
}

impl TryFrom<u8> for Land {
    type Error = &'static str;

    fn try_from(code: u8) -> Result<Self, Self::Error> {
        Land::from_code(code).ok_or("unknown land")
    }
}

impl From<Woods> for u8 {
    fn from(value: Woods) -> Self {
        value.code()
    }
}

impl TryFrom<u8> for Woods {
    type Error = &'static str;

    fn try_from(code: u8) -> Result<Self, Self::Error> {
        Woods::from_code(code).ok_or("unknown woods")
    }
}

impl From<StartOfDay> for u8 {
    fn from(value: StartOfDay) -> Self {
        value.code()
    }
}

impl TryFrom<u8> for StartOfDay {
    type Error = &'static str;

    fn try_from(code: u8) -> Result<Self, Self::Error> {
        StartOfDay::from_code(code).ok_or("unknown start of day")
    }
}

impl From<CropKind> for u8 {
    fn from(value: CropKind) -> Self {
        value.index() as u8
    }
}

impl TryFrom<u8> for CropKind {
    type Error = &'static str;

    fn try_from(code: u8) -> Result<Self, Self::Error> {
        CropKind::from_index(code).ok_or("unknown crop")
    }
}

impl From<Item> for [u8; 2] {
    fn from(value: Item) -> Self {
        value.code()
    }
}

impl TryFrom<[u8; 2]> for Item {
    type Error = &'static str;

    fn try_from(code: [u8; 2]) -> Result<Self, Self::Error> {
        Item::from_code(code).ok_or("unknown item")
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ask {
    Walk,
    Jump,
    SelectTarget,
    AbilityOne,
    AbilityTwo,
    AbilityThree,
    AbilityFour,
    AbilityFive,
    AbilitySix,
    Mine,
    Interact,
    Drop,
    PreviousSlot,
    NextSlot,
    Pocket,
    Map,
    Menu,
    Back,
    Confirm,
}

pub const ABILITY_ASKS: [Ask; 6] = [
    Ask::AbilityOne,
    Ask::AbilityTwo,
    Ask::AbilityThree,
    Ask::AbilityFour,
    Ask::AbilityFive,
    Ask::AbilitySix,
];

impl nightshade_input::Action for Ask {
    fn all() -> &'static [Self] {
        &[
            Self::Walk,
            Self::Jump,
            Self::SelectTarget,
            Self::AbilityOne,
            Self::AbilityTwo,
            Self::AbilityThree,
            Self::AbilityFour,
            Self::AbilityFive,
            Self::AbilitySix,
            Self::Mine,
            Self::Interact,
            Self::Drop,
            Self::PreviousSlot,
            Self::NextSlot,
            Self::Pocket,
            Self::Map,
            Self::Menu,
            Self::Back,
            Self::Confirm,
        ]
    }

    fn name(&self) -> &'static str {
        match self {
            Self::Walk => "walk",
            Self::Jump => "jump",
            Self::SelectTarget => "select target",
            Self::AbilityOne => "ability one",
            Self::AbilityTwo => "ability two",
            Self::AbilityThree => "ability three",
            Self::AbilityFour => "ability four",
            Self::AbilityFive => "ability five",
            Self::AbilitySix => "ability six",
            Self::Mine => "mine",
            Self::Interact => "interact",
            Self::Drop => "drop",
            Self::PreviousSlot => "previous slot",
            Self::NextSlot => "next slot",
            Self::Pocket => "pocket",
            Self::Map => "map",
            Self::Menu => "menu",
            Self::Back => "back",
            Self::Confirm => "confirm",
        }
    }

    fn default_bindings(&self) -> &'static [nightshade_input::Binding] {
        use nightshade_gamepad::contract::GamepadButton;
        use nightshade_input::{Binding, Pointer, Stick};
        use nightshade_platform::KeyCode;
        match self {
            Self::Walk => &[
                Binding::Keys {
                    up: KeyCode::KeyW,
                    down: KeyCode::KeyS,
                    left: KeyCode::KeyA,
                    right: KeyCode::KeyD,
                },
                Binding::Keys {
                    up: KeyCode::ArrowUp,
                    down: KeyCode::ArrowDown,
                    left: KeyCode::ArrowLeft,
                    right: KeyCode::ArrowRight,
                },
                Binding::Stick(Stick::Left),
            ],
            Self::Jump => &[
                Binding::Key(KeyCode::Space),
                Binding::Pad(GamepadButton::South),
            ],
            Self::SelectTarget => &[Binding::Key(KeyCode::Tab)],
            Self::AbilityOne => &[Binding::Key(KeyCode::Digit1)],
            Self::AbilityTwo => &[Binding::Key(KeyCode::Digit2)],
            Self::AbilityThree => &[Binding::Key(KeyCode::Digit3)],
            Self::AbilityFour => &[Binding::Key(KeyCode::Digit4)],
            Self::AbilityFive => &[Binding::Key(KeyCode::Digit5)],
            Self::AbilitySix => &[Binding::Key(KeyCode::Digit6)],
            Self::Mine => &[Binding::Pointer(Pointer::Left)],
            Self::Interact => &[Binding::Key(KeyCode::KeyE)],
            Self::Drop => &[Binding::Key(KeyCode::KeyQ)],
            Self::PreviousSlot => &[Binding::Key(KeyCode::BracketLeft)],
            Self::NextSlot => &[Binding::Key(KeyCode::BracketRight)],
            Self::Pocket => &[Binding::Key(KeyCode::KeyI)],
            Self::Map => &[Binding::Key(KeyCode::KeyM)],
            Self::Menu => &[
                Binding::Key(KeyCode::Escape),
                Binding::Pad(GamepadButton::Start),
            ],
            Self::Back => &[Binding::Key(KeyCode::Escape)],
            Self::Confirm => &[
                Binding::Key(KeyCode::Enter),
                Binding::Pad(GamepadButton::South),
            ],
        }
    }
}
