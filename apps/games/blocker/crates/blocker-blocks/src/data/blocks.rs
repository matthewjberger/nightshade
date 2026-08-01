use crate::data::tools::ToolKind;
use crate::theme;
use nalgebra_glm::{IVec3, Vec3};
use nightshade_voxels::prelude::{BlockLook, Blocks, Fill, Liquid, Part, Patch};
pub use nightshade_voxels::prelude::{Cell, Face, Shape};
use serde::{Deserialize, Serialize};

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
                            | BlockKind::SnowLayer
                    ),
            }
        })
        .collect();
    Blocks {
        looks,
        atlas_columns: crate::data::atlas::ATLAS_COLUMNS,
        atlas_rows: crate::data::atlas::ATLAS_ROWS,
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
            material: crate::data::atlas::WATER_MATERIAL.to_string(),
        }],
    }
}

fn water_fill(kind: BlockKind) -> Option<Fill> {
    let level = water_level(kind);
    (level > 0).then_some(Fill {
        liquid: crate::data::blocks::WATER,
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
    DoorOpenNorthMirrored,
    DoorOpenSouthMirrored,
    DoorOpenEastMirrored,
    DoorOpenWestMirrored,
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
    SnowLayer,
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
    pub footing: &'static [BlockKind],
}

pub const BLOCKS: [BlockInfo; 58] = [
    uniform(BlockKind::Air, "Air", (0, 0), 0.0, None).not_solid(),
    uniform(BlockKind::Bedrock, "Bedrock", (4, 3), f32::INFINITY, None),
    uniform(BlockKind::Stone, "Stone", (19, 6), 1.5, PICKAXE).drops(BlockKind::Cobblestone),
    uniform(BlockKind::Dirt, "Dirt", (8, 4), 0.5, SHOVEL).grows_crops(),
    uniform(BlockKind::Grass, "Grass", (3, 10), 0.6, SHOVEL)
        .faced((8, 4), (0, 10))
        .grows_crops()
        .drops(BlockKind::Dirt),
    uniform(BlockKind::Cobblestone, "Cobblestone", (2, 5), 2.0, PICKAXE),
    uniform(BlockKind::Sand, "Sand", (18, 7), 0.5, SHOVEL),
    uniform(BlockKind::Gravel, "Gravel", (4, 10), 0.6, SHOVEL),
    uniform(BlockKind::OakLog, "Oak Log", (13, 13), 2.0, AXE).faced((13, 13), (12, 13)),
    uniform(BlockKind::OakLeaves, "Oak Leaves", (15, 12), 0.2, None).cutout(),
    uniform(BlockKind::OakPlanks, "Oak Planks", (12, 15), 2.0, AXE),
    uniform(BlockKind::Glass, "Glass", theme::GLASS_CELL, 0.3, None)
        .transparent()
        .drops(BlockKind::Air),
    uniform(BlockKind::Bricks, "Bricks", (5, 3), 2.0, PICKAXE),
    uniform(BlockKind::Glowstone, "Glowstone", (13, 9), 0.3, None),
    uniform(BlockKind::CoalOre, "Coal Ore", (1, 5), 3.0, PICKAXE),
    uniform(BlockKind::IronOre, "Iron Ore", (15, 11), 3.0, PICKAXE),
    uniform(BlockKind::GoldOre, "Gold Ore", (15, 9), 3.0, PICKAXE),
    uniform(BlockKind::DiamondOre, "Diamond Ore", (8, 3), 3.0, PICKAXE),
    uniform(BlockKind::CraftingTable, "Crafting Table", (4, 6), 2.5, AXE).faced((12, 15), (12, 15)),
    uniform(BlockKind::Tnt, "TNT", (19, 13), 0.0, None).faced((19, 14), (19, 15)),
    uniform(BlockKind::Obsidian, "Obsidian", (4, 15), 12.0, PICKAXE),
    uniform(BlockKind::WoolRed, "Red Wool", (21, 13), 0.8, SWORD),
    uniform(BlockKind::WoolBlue, "Blue Wool", (21, 2), 0.8, SWORD),
    uniform(BlockKind::WoolWhite, "White Wool", (21, 15), 0.8, SWORD),
    plant(BlockKind::RedFlower, "Rose", (14, 2), 0.0, FLOWER_FOOTING),
    plant(
        BlockKind::YellowFlower,
        "Dandelion",
        (13, 6),
        0.0,
        FLOWER_FOOTING,
    ),
    liquid(BlockKind::Water),
    uniform(BlockKind::Campfire, "Campfire", (0, 0), 1.0, PICKAXE)
        .faced((2, 5), (2, 5))
        .glowing(CAMPFIRE_GLOW),
    plant(BlockKind::Signpost, "Signpost", (12, 13), 0.5, &[])
        .with_parts(&SIGN_PARTS)
        .shaped(Shape::Parts),
    torch_block(BlockKind::Torch, [0.0, 0.0, 0.0], &TORCH_PARTS),
    torch_block(BlockKind::TorchNorth, [0.0, 0.0, -1.0], &TORCH_NORTH_PARTS),
    torch_block(BlockKind::TorchSouth, [0.0, 0.0, 1.0], &TORCH_SOUTH_PARTS),
    torch_block(BlockKind::TorchEast, [1.0, 0.0, 0.0], &TORCH_EAST_PARTS),
    torch_block(BlockKind::TorchWest, [-1.0, 0.0, 0.0], &TORCH_WEST_PARTS),
    door_block(BlockKind::DoorNorth, NORTH, false, &DOOR_NORTH_PARTS),
    door_block(BlockKind::DoorSouth, SOUTH, false, &DOOR_SOUTH_PARTS),
    door_block(BlockKind::DoorEast, EAST, false, &DOOR_EAST_PARTS),
    door_block(BlockKind::DoorWest, WEST, false, &DOOR_WEST_PARTS),
    door_block(
        BlockKind::DoorOpenNorth,
        NORTH,
        true,
        &DOOR_OPEN_NORTH_PARTS,
    ),
    door_block(
        BlockKind::DoorOpenSouth,
        SOUTH,
        true,
        &DOOR_OPEN_SOUTH_PARTS,
    ),
    door_block(BlockKind::DoorOpenEast, EAST, true, &DOOR_OPEN_EAST_PARTS),
    door_block(BlockKind::DoorOpenWest, WEST, true, &DOOR_OPEN_WEST_PARTS),
    door_block(
        BlockKind::DoorOpenNorthMirrored,
        NORTH,
        true,
        &DOOR_OPEN_NORTH_MIRRORED_PARTS,
    ),
    door_block(
        BlockKind::DoorOpenSouthMirrored,
        SOUTH,
        true,
        &DOOR_OPEN_SOUTH_MIRRORED_PARTS,
    ),
    door_block(
        BlockKind::DoorOpenEastMirrored,
        EAST,
        true,
        &DOOR_OPEN_EAST_MIRRORED_PARTS,
    ),
    door_block(
        BlockKind::DoorOpenWestMirrored,
        WEST,
        true,
        &DOOR_OPEN_WEST_MIRRORED_PARTS,
    ),
    liquid(BlockKind::WaterFour),
    liquid(BlockKind::WaterThree),
    liquid(BlockKind::WaterTwo),
    liquid(BlockKind::WaterOne),
    uniform(BlockKind::Chest, "Chest", theme::CHEST_TOP_CELL, 2.0, AXE)
        .faced(theme::CHEST_TOP_CELL, theme::CHEST_SIDE_CELL),
    uniform(BlockKind::Snow, "Snow", (19, 3), 0.2, SHOVEL),
    uniform(BlockKind::SnowGrass, "Snowy Grass", (19, 3), 0.6, SHOVEL)
        .faced((8, 4), (2, 10))
        .drops(BlockKind::Dirt),
    uniform(BlockKind::Ice, "Ice", (15, 3), 0.5, PICKAXE)
        .cutout()
        .drops(BlockKind::Air),
    uniform(BlockKind::RedSand, "Red Sand", (5, 10), 0.5, SHOVEL),
    uniform(BlockKind::Terracotta, "Terracotta", (5, 11), 1.25, PICKAXE),
    plant(BlockKind::DeadBush, "Dead Bush", (19, 2), 0.0, BUSH_FOOTING),
    uniform(BlockKind::SnowLayer, "Snow Layer", (19, 3), 0.1, SHOVEL)
        .with_parts(&SNOW_LAYER_PARTS)
        .shaped(Shape::Parts)
        .not_solid()
        .drops(BlockKind::Air),
];

const PICKAXE: Option<ToolKind> = Some(ToolKind::Pickaxe);

const SHOVEL: Option<ToolKind> = Some(ToolKind::Shovel);

const AXE: Option<ToolKind> = Some(ToolKind::Axe);

const SWORD: Option<ToolKind> = Some(ToolKind::Sword);

const CAMPFIRE_GLOW: (Vec3, f32, f32) = (
    theme::CAMPFIRE_COLOR,
    theme::CAMPFIRE_STRENGTH,
    theme::CAMPFIRE_RANGE,
);

const TORCH_GLOW: (Vec3, f32, f32) = (
    theme::TORCH_COLOR,
    theme::TORCH_STRENGTH,
    theme::TORCH_RANGE,
);

const fn blocks_line_up() -> bool {
    let mut index = 0;
    while index < BLOCKS.len() {
        if BLOCKS[index].kind as usize != index {
            return false;
        }
        index += 1;
    }
    true
}

const _: () = assert!(
    blocks_line_up(),
    "a BLOCKS row does not sit at the index of its BlockKind"
);

const fn uniform(
    kind: BlockKind,
    name: &'static str,
    cell: Cell,
    hardness: f32,
    tool: Option<ToolKind>,
) -> BlockInfo {
    BlockInfo {
        kind,
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
        shape: Shape::Cube,
        cutout: false,
        transparent: false,
        solid: true,
        hardness,
        tool,
        drop: None,
        footing: &[],
    }
}

const fn plant(
    kind: BlockKind,
    name: &'static str,
    cell: Cell,
    hardness: f32,
    footing: &'static [BlockKind],
) -> BlockInfo {
    uniform(kind, name, cell, hardness, None)
        .shaped(Shape::Cross)
        .cutout()
        .not_solid()
        .footed(footing)
}

const fn liquid(kind: BlockKind) -> BlockInfo {
    uniform(kind, "Water", (20, 6), f32::INFINITY, None)
        .shaped(Shape::Liquid)
        .not_solid()
}

const fn torch_block(kind: BlockKind, lean: [f32; 3], parts: &'static [Part]) -> BlockInfo {
    let row = plant(kind, "Torch", (20, 1), 0.0, &[])
        .glowing(TORCH_GLOW)
        .with_parts(parts)
        .shaped(Shape::Parts)
        .drops(BlockKind::Torch)
        .leaning(Vec3::new(lean[0], lean[1], lean[2]));
    if lean[0] != 0.0 || lean[2] != 0.0 {
        row.on_wall()
    } else {
        row
    }
}

const fn door_block(
    kind: BlockKind,
    facing: [i32; 3],
    open: bool,
    parts: &'static [Part],
) -> BlockInfo {
    let row = uniform(kind, "Door", DOOR_LOWER, 2.0, AXE)
        .as_door(IVec3::new(facing[0], facing[1], facing[2]), open)
        .with_parts(parts)
        .shaped(Shape::Parts)
        .cutout()
        .drops(BlockKind::DoorNorth);
    if open { row.not_solid() } else { row }
}

impl BlockInfo {
    const fn faced(mut self, bottom: Cell, side: Cell) -> Self {
        self.bottom = bottom;
        self.side = side;
        self
    }

    const fn drops(mut self, kind: BlockKind) -> Self {
        self.drop = Some(kind);
        self
    }

    const fn shaped(mut self, shape: Shape) -> Self {
        self.shape = shape;
        self
    }

    const fn with_parts(mut self, parts: &'static [Part]) -> Self {
        self.parts = parts;
        self
    }

    const fn cutout(mut self) -> Self {
        self.cutout = true;
        self
    }

    const fn not_solid(mut self) -> Self {
        self.solid = false;
        self
    }

    const fn as_door(mut self, facing: IVec3, open: bool) -> Self {
        self.door = Some((facing, open));
        self
    }

    const fn glowing(mut self, glow: (Vec3, f32, f32)) -> Self {
        self.glow = Some(glow);
        self
    }

    const fn on_wall(mut self) -> Self {
        self.on_wall = true;
        self
    }

    const fn leaning(mut self, lean: Vec3) -> Self {
        self.lean = lean;
        self
    }

    const fn grows_crops(mut self) -> Self {
        self.grows_crops = true;
        self
    }

    const fn transparent(mut self) -> Self {
        self.transparent = true;
        self
    }

    const fn footed(mut self, footing: &'static [BlockKind]) -> Self {
        self.footing = footing;
        self
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

    pub fn door_mirrored_kind(facing: IVec3) -> BlockKind {
        match (facing.x, facing.z) {
            (0, -1) => BlockKind::DoorOpenNorthMirrored,
            (0, 1) => BlockKind::DoorOpenSouthMirrored,
            (1, 0) => BlockKind::DoorOpenEastMirrored,
            _ => BlockKind::DoorOpenWestMirrored,
        }
    }

    pub fn door_swing(facing: IVec3, mirrored: bool) -> IVec3 {
        if mirrored {
            IVec3::new(facing.z, 0, -facing.x)
        } else {
            IVec3::new(-facing.z, 0, facing.x)
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

pub const FLOWER_FOOTING: &[BlockKind] = &[BlockKind::Grass];
pub const BUSH_FOOTING: &[BlockKind] = &[
    BlockKind::Grass,
    BlockKind::Sand,
    BlockKind::RedSand,
    BlockKind::Terracotta,
    BlockKind::SnowGrass,
];

pub const DOOR_LOWER: Cell = (9, 3);

pub const DOOR_UPPER: Cell = (9, 4);

const TEXEL: f32 = 1.0 / crate::data::atlas::TILE_TEXELS as f32;
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
    let rise = if wall {
        crate::data::blocks::TORCH_WALL_RISE
    } else {
        0.0
    };
    let base_x = 0.5 - lean[0] * crate::data::blocks::TORCH_WALL_SHIFT;
    let base_z = 0.5 - lean[2] * crate::data::blocks::TORCH_WALL_SHIFT;
    Part {
        low: [base_x - TORCH_HALF_WIDTH, rise, base_z - TORCH_HALF_WIDTH],
        high: [
            base_x + TORCH_HALF_WIDTH,
            rise + TORCH_HEIGHT,
            base_z + TORCH_HALF_WIDTH,
        ],
        lift: [
            lean[0] * crate::data::blocks::TORCH_WALL_TILT,
            0.0,
            lean[2] * crate::data::blocks::TORCH_WALL_TILT,
        ],
        side: patch(7.0, 6.0, 9.0, 16.0),
        top: patch(7.0, 6.0, 9.0, 8.0),
        bottom: patch(7.0, 6.0, 9.0, 8.0),
    }
}

const fn door(facing: [i32; 3], open: bool, mirrored: bool) -> Part {
    let side = if !open {
        facing
    } else if mirrored {
        [facing[2], 0, -facing[0]]
    } else {
        [-facing[2], 0, facing[0]]
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
const DOOR_NORTH_PARTS: [Part; 1] = [door(NORTH, false, false)];
const DOOR_SOUTH_PARTS: [Part; 1] = [door(SOUTH, false, false)];
const DOOR_EAST_PARTS: [Part; 1] = [door(EAST, false, false)];
const DOOR_WEST_PARTS: [Part; 1] = [door(WEST, false, false)];
const DOOR_OPEN_NORTH_PARTS: [Part; 1] = [door(NORTH, true, false)];
const DOOR_OPEN_SOUTH_PARTS: [Part; 1] = [door(SOUTH, true, false)];
const DOOR_OPEN_EAST_PARTS: [Part; 1] = [door(EAST, true, false)];
const DOOR_OPEN_WEST_PARTS: [Part; 1] = [door(WEST, true, false)];
const DOOR_OPEN_NORTH_MIRRORED_PARTS: [Part; 1] = [door(NORTH, true, true)];
const DOOR_OPEN_SOUTH_MIRRORED_PARTS: [Part; 1] = [door(SOUTH, true, true)];
const DOOR_OPEN_EAST_MIRRORED_PARTS: [Part; 1] = [door(EAST, true, true)];
const DOOR_OPEN_WEST_MIRRORED_PARTS: [Part; 1] = [door(WEST, true, true)];
const SNOW_LAYER_PARTS: [Part; 1] = [Part {
    low: [0.0, 0.0, 0.0],
    high: [1.0, 2.0 * TEXEL, 1.0],
    lift: [0.0, 0.0, 0.0],
    side: patch(0.0, 14.0, 16.0, 16.0),
    top: patch(0.0, 0.0, 16.0, 16.0),
    bottom: patch(0.0, 0.0, 16.0, 16.0),
}];

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
pub enum Palette {
    #[default]
    Kenney,
}

impl Palette {
    pub fn bytes(self) -> &'static [u8] {
        match self {
            Palette::Kenney => crate::data::atlas::KENNEY_ATLAS_BYTES,
        }
    }

    pub fn tinted_cells(self) -> &'static [(Cell, [f32; 3])] {
        match self {
            Palette::Kenney => &[],
        }
    }

    pub fn code(self) -> u8 {
        match self {
            Palette::Kenney => 0,
        }
    }

    pub fn from_code(code: u8) -> Option<Palette> {
        match code {
            0 => Some(Palette::Kenney),
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

pub const WATER: u8 = 0;

pub const TORCH_WALL_RISE: f32 = 0.22;

pub const TORCH_WALL_SHIFT: f32 = 0.42;

pub const TORCH_WALL_TILT: f32 = 0.2;

pub const TORCH_FLAME_HEIGHT: f32 = 0.66;
