use crate::theme;
use nalgebra_glm::{IVec3, Vec2, Vec3, vec2};
use nightshade_voxels::prelude::{BlockLook, Blocks, Fill, Liquid, Part, Patch};
pub use nightshade_voxels::prelude::{Cell, Face, Shape};
use serde::{Deserialize, Serialize};

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
        ..uniform("Glass", theme::GLASS_CELL, 0.3, None)
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
        cutout: true,
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

pub const TINTED_CELLS: [(Cell, [f32; 3]); 2] =
    [((3, 10), theme::GRASS_TINT), ((15, 12), theme::LEAF_TINT)];

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
#[serde(into = "u8", try_from = "u8")]
pub enum Palette {
    #[default]
    Classic,
    Kenney,
}

impl Palette {
    pub fn name(self) -> &'static str {
        match self {
            Palette::Classic => "Classic",
            Palette::Kenney => "Kenney",
        }
    }

    pub fn bytes(self) -> &'static [u8] {
        match self {
            Palette::Classic => theme::ATLAS_BYTES,
            Palette::Kenney => theme::KENNEY_ATLAS_BYTES,
        }
    }

    pub fn tinted_cells(self) -> &'static [(Cell, [f32; 3])] {
        match self {
            Palette::Classic => &TINTED_CELLS,
            Palette::Kenney => &[],
        }
    }

    pub fn next(self) -> Palette {
        match self {
            Palette::Classic => Palette::Kenney,
            Palette::Kenney => Palette::Classic,
        }
    }

    pub fn code(self) -> u8 {
        match self {
            Palette::Classic => 0,
            Palette::Kenney => 1,
        }
    }

    pub fn from_code(code: u8) -> Option<Palette> {
        match code {
            0 => Some(Palette::Classic),
            1 => Some(Palette::Kenney),
            _ => None,
        }
    }
}

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

impl Land {
    pub fn name(self) -> &'static str {
        match self {
            Land::Rolling => "Rolling",
            Land::Flat => "Flat",
            Land::Islands => "Islands",
            Land::Peaks => "Peaks",
            Land::Extreme => "Extreme",
        }
    }

    pub fn told(self) -> &'static str {
        match self {
            Land::Rolling => "Hills, shores and open ground",
            Land::Flat => "One level plain to build on",
            Land::Islands => "Small land in a wide sea",
            Land::Peaks => "Tall ground and deep valleys",
            Land::Extreme => "Cliffs, arches and stone that leans over you",
        }
    }

    pub fn next(self) -> Land {
        match self {
            Land::Rolling => Land::Flat,
            Land::Flat => Land::Islands,
            Land::Islands => Land::Peaks,
            Land::Peaks => Land::Extreme,
            Land::Extreme => Land::Rolling,
        }
    }

    pub fn rock(self) -> f32 {
        match self {
            Land::Rolling => 1.0,
            Land::Flat => 0.0,
            Land::Islands => 0.9,
            Land::Peaks => 1.5,
            Land::Extreme => 2.0,
        }
    }

    pub fn headroom(self) -> i32 {
        match self {
            Land::Rolling => 10,
            Land::Flat => 10,
            Land::Islands => 10,
            Land::Peaks => 16,
            Land::Extreme => 28,
        }
    }

    pub fn lift(self) -> f32 {
        match self {
            Land::Rolling => 0.55,
            Land::Flat => 0.0,
            Land::Islands => 0.6,
            Land::Peaks => 0.75,
            Land::Extreme => 1.05,
        }
    }

    pub fn code(self) -> u8 {
        match self {
            Land::Rolling => 0,
            Land::Flat => 1,
            Land::Islands => 2,
            Land::Peaks => 3,
            Land::Extreme => 4,
        }
    }

    pub fn from_code(code: u8) -> Option<Land> {
        match code {
            0 => Some(Land::Rolling),
            1 => Some(Land::Flat),
            2 => Some(Land::Islands),
            3 => Some(Land::Peaks),
            4 => Some(Land::Extreme),
            _ => None,
        }
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

impl Woods {
    pub fn name(self) -> &'static str {
        match self {
            Woods::Bare => "None",
            Woods::Few => "Few",
            Woods::Normal => "Normal",
            Woods::Thick => "Thick",
        }
    }

    pub fn scale(self) -> f32 {
        match self {
            Woods::Bare => 0.0,
            Woods::Few => 0.35,
            Woods::Normal => 1.0,
            Woods::Thick => 2.4,
        }
    }

    pub fn next(self) -> Woods {
        match self {
            Woods::Bare => Woods::Few,
            Woods::Few => Woods::Normal,
            Woods::Normal => Woods::Thick,
            Woods::Thick => Woods::Bare,
        }
    }

    pub fn code(self) -> u8 {
        match self {
            Woods::Bare => 0,
            Woods::Few => 1,
            Woods::Normal => 2,
            Woods::Thick => 3,
        }
    }

    pub fn from_code(code: u8) -> Option<Woods> {
        match code {
            0 => Some(Woods::Bare),
            1 => Some(Woods::Few),
            2 => Some(Woods::Normal),
            3 => Some(Woods::Thick),
            _ => None,
        }
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

impl StartOfDay {
    pub fn name(self) -> &'static str {
        match self {
            StartOfDay::Morning => "Morning",
            StartOfDay::Noon => "Noon",
            StartOfDay::Evening => "Evening",
            StartOfDay::Night => "Night",
        }
    }

    pub fn hour(self) -> f32 {
        match self {
            StartOfDay::Morning => theme::START_HOUR,
            StartOfDay::Noon => 12.0,
            StartOfDay::Evening => 17.5,
            StartOfDay::Night => 22.0,
        }
    }

    pub fn next(self) -> StartOfDay {
        match self {
            StartOfDay::Morning => StartOfDay::Noon,
            StartOfDay::Noon => StartOfDay::Evening,
            StartOfDay::Evening => StartOfDay::Night,
            StartOfDay::Night => StartOfDay::Morning,
        }
    }

    pub fn code(self) -> u8 {
        match self {
            StartOfDay::Morning => 0,
            StartOfDay::Noon => 1,
            StartOfDay::Evening => 2,
            StartOfDay::Night => 3,
        }
    }

    pub fn from_code(code: u8) -> Option<StartOfDay> {
        match code {
            0 => Some(StartOfDay::Morning),
            1 => Some(StartOfDay::Noon),
            2 => Some(StartOfDay::Evening),
            3 => Some(StartOfDay::Night),
            _ => None,
        }
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

pub const CLASSIC_ART: [((u32, u32), [&str; 16]); 1] = [(
    theme::GLASS_CELL,
    [
        "iiiiiiiiiiiiiiii",
        "i..............i",
        "i...w..........i",
        "i..w...........i",
        "i..............i",
        "i..........ww..i",
        "i..........w...i",
        "i..............i",
        "i..............i",
        "i...ww.........i",
        "i....w.........i",
        "i..............i",
        "i.........w....i",
        "i........ww....i",
        "i..............i",
        "iiiiiiiiiiiiiiii",
    ],
)];

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
    pub feeds: f32,
    pub name: &'static str,
    pub seed_name: &'static str,
    pub stages: [u32; 4],
    pub grow_seconds: f32,
}

pub const CROPS: [CropInfo; 2] = [
    CropInfo {
        kind: CropKind::Carrot,
        feeds: theme::CARROT_FOOD,
        name: "Carrot",
        seed_name: "Carrot Seeds",
        stages: [5, 6, 7, 8],
        grow_seconds: 50.0,
    },
    CropInfo {
        kind: CropKind::Wheat,
        feeds: theme::WHEAT_FOOD,
        name: "Wheat",
        seed_name: "Wheat Seeds",
        stages: [64, 65, 66, 67],
        grow_seconds: 40.0,
    },
];

impl CropKind {
    pub fn feeds(self) -> f32 {
        CROPS[self as usize].feeds
    }

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
    Boat,
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
            Item::Boat => [7, 0],
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
            7 => Some(Item::Boat),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Item::Block(kind) => kind.info().name,
            Item::Tool(tool) => tool.name(),
            Item::Firework => "Firework",
            Item::Boat => "Boat",
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
            Item::Boat => Some(11),
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
            Item::Boat => Some([
                "................",
                "................",
                "................",
                "................",
                ".....TT.........",
                ".....TT.........",
                ".B...TT......B..",
                ".BB..TT.....BB..",
                ".BbB.TT....BbB..",
                ".BbbBBBBBBBbbB..",
                ".BbbbbbbbbbbbB..",
                ".BBbbbbbbbbbBB..",
                "..BBBBBBBBBBB...",
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

pub const ART_ITEMS: [Item; 12] = [
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
    Item::Boat,
];

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum MonsterKind {
    #[default]
    Slime,
    Bat,
    Spider,
    Skeleton,
    Ghost,
}

#[derive(Clone, Copy, Debug)]
pub struct MonsterInfo {
    pub name: &'static str,
    pub tile: u32,
    pub health: f32,
    pub bite: f32,
    pub speed: f32,
    pub width: f32,
    pub floats: bool,
}

impl MonsterKind {
    pub const ALL: [MonsterKind; 5] = [
        MonsterKind::Slime,
        MonsterKind::Bat,
        MonsterKind::Spider,
        MonsterKind::Skeleton,
        MonsterKind::Ghost,
    ];

    pub fn info(self) -> MonsterInfo {
        match self {
            MonsterKind::Slime => MonsterInfo {
                name: "Slime",
                tile: theme::SLIME_TILE,
                health: 4.0,
                bite: 1.0,
                speed: 1.1,
                width: 0.8,
                floats: false,
            },
            MonsterKind::Bat => MonsterInfo {
                name: "Bat",
                tile: theme::BAT_TILE,
                health: 2.0,
                bite: 0.5,
                speed: 2.4,
                width: 0.7,
                floats: true,
            },
            MonsterKind::Spider => MonsterInfo {
                name: "Spider",
                tile: theme::SPIDER_TILE,
                health: 5.0,
                bite: 1.5,
                speed: 2.0,
                width: 1.0,
                floats: false,
            },
            MonsterKind::Skeleton => MonsterInfo {
                name: "Skeleton",
                tile: theme::SKELETON_TILE,
                health: 8.0,
                bite: 2.0,
                speed: 1.5,
                width: 1.0,
                floats: false,
            },
            MonsterKind::Ghost => MonsterInfo {
                name: "Ghost",
                tile: theme::GHOST_TILE,
                health: 6.0,
                bite: 2.5,
                speed: 1.3,
                width: 0.9,
                floats: true,
            },
        }
    }
}

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
        stack(Item::Boat, 1),
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

pub const RECIPES: [Recipe; 17] = [
    Recipe {
        output: Item::Boat,
        count: 1,
        inputs: &[(Item::Block(BlockKind::OakPlanks), 5)],
    },
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

pub const SHOP: [ShopRow; 13] = [
    ShopRow {
        item: Item::Block(BlockKind::Torch),
        buy: Some(1),
        sell: None,
    },
    ShopRow {
        item: Item::Firework,
        buy: Some(4),
        sell: None,
    },
    ShopRow {
        item: Item::Seed(CropKind::Carrot),
        buy: Some(2),
        sell: None,
    },
    ShopRow {
        item: Item::Seed(CropKind::Wheat),
        buy: Some(1),
        sell: None,
    },
    ShopRow {
        item: Item::Block(BlockKind::Glowstone),
        buy: Some(6),
        sell: None,
    },
    ShopRow {
        item: Item::Block(BlockKind::Tnt),
        buy: Some(8),
        sell: None,
    },
    ShopRow {
        item: Item::Crop(CropKind::Carrot),
        buy: None,
        sell: Some(3),
    },
    ShopRow {
        item: Item::Crop(CropKind::Wheat),
        buy: None,
        sell: Some(2),
    },
    ShopRow {
        item: Item::Block(BlockKind::OakLog),
        buy: None,
        sell: Some(1),
    },
    ShopRow {
        item: Item::Block(BlockKind::CoalOre),
        buy: None,
        sell: Some(2),
    },
    ShopRow {
        item: Item::Block(BlockKind::IronOre),
        buy: None,
        sell: Some(4),
    },
    ShopRow {
        item: Item::Block(BlockKind::GoldOre),
        buy: None,
        sell: Some(8),
    },
    ShopRow {
        item: Item::Block(BlockKind::DiamondOre),
        buy: None,
        sell: Some(20),
    },
];

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum NpcRole {
    Shopkeeper,
    #[default]
    Villager,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum AfterTalk {
    #[default]
    Nothing,
    OpenShop,
}

pub fn dialogue_for(role: NpcRole, coins: u32) -> (Vec<String>, AfterTalk) {
    match role {
        NpcRole::Shopkeeper => (
            vec![
                format!("Welcome to {}. I buy what the ground gives and sell what the sky wants.", theme::VILLAGE_NAME),
                format!("You carry {coins} coins. Fireworks are four apiece, and seeds are cheap."),
                "Have a look at the shelves.".to_string(),
            ],
            AfterTalk::OpenShop,
        ),
        NpcRole::Villager => (
            vec![
                "Plant seeds on the dirt and they grow on their own. Press E over a ripe one to pick it.".to_string(),
                "Bram at the shop pays coins for carrots, wheat, and ore. He sells fireworks too.".to_string(),
                "A campfire keeps the dark away. Craft one at the table with cobblestone and logs.".to_string(),
            ],
            AfterTalk::Nothing,
        ),
    }
}

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

    pub fn name(self) -> &'static str {
        match self {
            Biome::Plains => "plains",
            Biome::Forest => "forest",
            Biome::Desert => "desert",
            Biome::Hills => "hills",
            Biome::Ocean => "ocean",
            Biome::Snowfield => "snowfield",
            Biome::Peaks => "peaks",
            Biome::Swamp => "swamp",
            Biome::Badlands => "badlands",
        }
    }

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
    pub name: &'static str,
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
        '.' => Some(BlockKind::Air),
        _ => None,
    }
}

const SMALL_HOME: Building = Building {
    name: "small home",
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
    name: "long home",
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
    name: "shop",
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
    name: "well",
    layers: &[
        &["CCCC", "CUUC", "CUUC", "CCCC"],
        &["C..C", "....", "....", "C..C"],
        &["L..L", "....", "....", "L..L"],
        &["L..L", "....", "....", "L..L"],
        &["PPPP", "PPPP", "PPPP", "PPPP"],
    ],
};

const FIRE_PIT: Building = Building {
    name: "fire pit",
    layers: &[&["CCC", "CFC", "CCC"]],
};

pub const VILLAGE: [Building; 8] = [
    WELL, SHOP_HOUSE, SMALL_HOME, FIRE_PIT, LONG_HOME, SMALL_HOME, SMALL_HOME, FIRE_PIT,
];

impl From<Palette> for u8 {
    fn from(value: Palette) -> Self {
        value.code()
    }
}

impl TryFrom<u8> for Palette {
    type Error = &'static str;

    fn try_from(code: u8) -> Result<Self, Self::Error> {
        Palette::from_code(code).ok_or("unknown palette")
    }
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
    Move,
    Look,
    Jump,
    Sink,
    Sprint,
    Fly,
    Watch,
    Slot1,
    Slot2,
    Slot3,
    Slot4,
    Slot5,
    Slot6,
    Slot7,
    Slot8,
    Slot9,
    PreviousSlot,
    NextSlot,
    Mine,
    Place,
    Drop,
    Act,
    Swing,
    Pick,
    Lock,
    Unlock,
    Back,
    Pocket,
    Advance,
}

impl Ask {
    pub const SLOTS: [Ask; 9] = [
        Ask::Slot1,
        Ask::Slot2,
        Ask::Slot3,
        Ask::Slot4,
        Ask::Slot5,
        Ask::Slot6,
        Ask::Slot7,
        Ask::Slot8,
        Ask::Slot9,
    ];
}

impl nightshade_input::Action for Ask {
    fn all() -> &'static [Self] {
        &[
            Self::Move,
            Self::Look,
            Self::Jump,
            Self::Sink,
            Self::Sprint,
            Self::Fly,
            Self::Watch,
            Self::Slot1,
            Self::Slot2,
            Self::Slot3,
            Self::Slot4,
            Self::Slot5,
            Self::Slot6,
            Self::Slot7,
            Self::Slot8,
            Self::Slot9,
            Self::PreviousSlot,
            Self::NextSlot,
            Self::Mine,
            Self::Place,
            Self::Drop,
            Self::Act,
            Self::Swing,
            Self::Pick,
            Self::Lock,
            Self::Unlock,
            Self::Back,
            Self::Pocket,
            Self::Advance,
        ]
    }

    fn name(&self) -> &'static str {
        match self {
            Self::Move => "move",
            Self::Look => "look",
            Self::Jump => "jump",
            Self::Sink => "sink",
            Self::Sprint => "sprint",
            Self::Fly => "fly",
            Self::Watch => "watch",
            Self::Slot1 => "slot 1",
            Self::Slot2 => "slot 2",
            Self::Slot3 => "slot 3",
            Self::Slot4 => "slot 4",
            Self::Slot5 => "slot 5",
            Self::Slot6 => "slot 6",
            Self::Slot7 => "slot 7",
            Self::Slot8 => "slot 8",
            Self::Slot9 => "slot 9",
            Self::PreviousSlot => "previous slot",
            Self::NextSlot => "next slot",
            Self::Mine => "mine",
            Self::Place => "place",
            Self::Drop => "drop",
            Self::Act => "act",
            Self::Swing => "swing",
            Self::Pick => "pick",
            Self::Lock => "lock",
            Self::Unlock => "unlock",
            Self::Back => "back",
            Self::Pocket => "pocket",
            Self::Advance => "advance",
        }
    }

    fn default_bindings(&self) -> &'static [nightshade_input::Binding] {
        use nightshade_gamepad::contract::{GamepadAxis, GamepadButton};
        use nightshade_input::{Binding, Pointer, Stick};
        use nightshade_platform::KeyCode;
        match self {
            Self::Move => &[
                Binding::Keys {
                    up: KeyCode::KeyW,
                    down: KeyCode::KeyS,
                    left: KeyCode::KeyA,
                    right: KeyCode::KeyD,
                },
                Binding::Stick(Stick::Left),
            ],
            Self::Look => &[Binding::Stick(Stick::Right)],
            Self::Jump => &[
                Binding::Key(KeyCode::Space),
                Binding::Pad(GamepadButton::South),
            ],
            Self::Sink => &[
                Binding::Key(KeyCode::ShiftLeft),
                Binding::Pad(GamepadButton::East),
            ],
            Self::Sprint => &[
                Binding::Key(KeyCode::ControlLeft),
                Binding::Pad(GamepadButton::LeftThumb),
            ],
            Self::Fly => &[
                Binding::Key(KeyCode::KeyF),
                Binding::Pad(GamepadButton::DPadUp),
            ],
            Self::Watch => &[
                Binding::Key(KeyCode::KeyG),
                Binding::Pad(GamepadButton::DPadDown),
            ],
            Self::Slot1 => &[Binding::Key(KeyCode::Digit1)],
            Self::Slot2 => &[Binding::Key(KeyCode::Digit2)],
            Self::Slot3 => &[Binding::Key(KeyCode::Digit3)],
            Self::Slot4 => &[Binding::Key(KeyCode::Digit4)],
            Self::Slot5 => &[Binding::Key(KeyCode::Digit5)],
            Self::Slot6 => &[Binding::Key(KeyCode::Digit6)],
            Self::Slot7 => &[Binding::Key(KeyCode::Digit7)],
            Self::Slot8 => &[Binding::Key(KeyCode::Digit8)],
            Self::Slot9 => &[Binding::Key(KeyCode::Digit9)],
            Self::PreviousSlot => &[Binding::Pad(GamepadButton::LeftBumper)],
            Self::NextSlot => &[Binding::Pad(GamepadButton::RightBumper)],
            Self::Mine => &[
                Binding::Pointer(Pointer::Left),
                Binding::Trigger(GamepadAxis::RightTrigger),
            ],
            Self::Place => &[
                Binding::Pointer(Pointer::Right),
                Binding::Pad(GamepadButton::LeftTrigger),
            ],
            Self::Drop => &[
                Binding::Key(KeyCode::KeyQ),
                Binding::Pad(GamepadButton::North),
            ],
            Self::Act => &[
                Binding::Key(KeyCode::KeyE),
                Binding::Pad(GamepadButton::West),
            ],
            Self::Swing => &[
                Binding::Pointer(Pointer::Left),
                Binding::Pad(GamepadButton::East),
            ],
            Self::Pick => &[Binding::Pointer(Pointer::Left)],
            Self::Lock => &[Binding::Pointer(Pointer::Left)],
            Self::Unlock => &[Binding::Key(KeyCode::Escape)],
            Self::Back => &[
                Binding::Key(KeyCode::Escape),
                Binding::Pad(GamepadButton::East),
            ],
            Self::Pocket => &[
                Binding::Key(KeyCode::KeyI),
                Binding::Key(KeyCode::Tab),
                Binding::Pad(GamepadButton::Start),
            ],
            Self::Advance => &[
                Binding::Key(KeyCode::KeyE),
                Binding::Key(KeyCode::Space),
                Binding::Key(KeyCode::Enter),
            ],
        }
    }
}
