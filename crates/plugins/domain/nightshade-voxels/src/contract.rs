use nalgebra_glm::{IVec3, Vec3};
use std::collections::HashMap;

pub type Cell = (u32, u32);

pub type ChunkKey = [i32; 3];

pub type ColumnKey = [i32; 2];

pub type SlabKey = [i32; 3];

pub const AIR: u8 = 0;

pub const FULL_LEVEL: u8 = 5;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Hit {
    pub block: IVec3,
    pub normal: IVec3,
    pub kind: u8,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Shape {
    #[default]
    Cube,
    Cross,
    Liquid,
    Parts,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Patch {
    pub low: [f32; 2],
    pub high: [f32; 2],
}

impl Patch {
    pub const FULL: Patch = Patch {
        low: [0.0, 0.0],
        high: [1.0, 1.0],
    };
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Part {
    pub low: [f32; 3],
    pub high: [f32; 3],
    pub lift: [f32; 3],
    pub side: Patch,
    pub top: Patch,
    pub bottom: Patch,
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ChunkLayer {
    #[default]
    Solid,
    Cutout,
    Transparent,
    Liquid(u8),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Fill {
    pub liquid: u8,
    pub level: u8,
}

#[derive(Clone, Debug)]
pub struct Liquid {
    pub levels: [u8; 6],
    pub reach: u8,
    pub pace: u8,
    pub material: String,
}

#[derive(Clone, Copy, Debug)]
pub struct BlockLook {
    pub top: Cell,
    pub bottom: Cell,
    pub side: Cell,
    pub shape: Shape,
    pub cutout: bool,
    pub transparent: bool,
    pub occludes: bool,
    pub fill: Option<Fill>,
    pub parts: &'static [Part],
    pub stacked: Option<Cell>,
    pub tall: bool,
    pub support: Option<[i32; 3]>,
    pub marked: bool,
    pub solid: bool,
    pub falls: bool,
    pub replaceable: bool,
    pub swung: Option<u8>,
}

impl Default for BlockLook {
    fn default() -> Self {
        Self {
            top: (0, 0),
            bottom: (0, 0),
            side: (0, 0),
            shape: Shape::Cube,
            cutout: false,
            transparent: false,
            occludes: true,
            fill: None,
            parts: &[],
            stacked: None,
            tall: false,
            support: None,
            marked: false,
            solid: true,
            falls: false,
            replaceable: false,
            swung: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Blocks {
    pub looks: Vec<BlockLook>,
    pub atlas_columns: u32,
    pub atlas_rows: u32,
    pub liquids: Vec<Liquid>,
}

impl Default for Blocks {
    fn default() -> Self {
        Self {
            looks: vec![BlockLook {
                occludes: false,
                solid: false,
                replaceable: true,
                ..Default::default()
            }],
            atlas_columns: 1,
            atlas_rows: 1,
            liquids: Vec::new(),
        }
    }
}

impl Blocks {
    pub fn look(&self, block: u8) -> BlockLook {
        self.looks
            .get(block as usize)
            .copied()
            .unwrap_or_else(|| self.looks[0])
    }

    pub fn occludes(&self, block: u8) -> bool {
        self.look(block).occludes
    }

    pub fn fill(&self, block: u8) -> Option<Fill> {
        self.look(block).fill
    }

    pub fn level(&self, block: u8) -> u8 {
        self.look(block).fill.map(|fill| fill.level).unwrap_or(0)
    }

    pub fn tall(&self, block: u8) -> bool {
        self.look(block).tall
    }

    pub fn is_liquid(&self, block: u8) -> bool {
        self.look(block).fill.is_some()
    }

    pub fn solid(&self, block: u8) -> bool {
        self.look(block).solid
    }

    pub fn falls(&self, block: u8) -> bool {
        self.look(block).falls
    }

    pub fn replaceable(&self, block: u8) -> bool {
        self.look(block).replaceable
    }

    pub fn support(&self, block: u8) -> Option<[i32; 3]> {
        self.look(block).support
    }

    pub fn liquid_block(&self, liquid: u8, level: u8) -> u8 {
        self.liquids
            .get(liquid as usize)
            .and_then(|held| held.levels.get(level as usize))
            .copied()
            .unwrap_or(AIR)
    }

    pub fn layers(&self) -> impl Iterator<Item = ChunkLayer> {
        [
            ChunkLayer::Solid,
            ChunkLayer::Cutout,
            ChunkLayer::Transparent,
        ]
        .into_iter()
        .chain((0..self.liquids.len() as u8).map(ChunkLayer::Liquid))
    }

    pub fn cell_index(&self, cell: Cell) -> f32 {
        (cell.1 * self.atlas_columns + cell.0) as f32
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Face {
    Top,
    Bottom,
    North,
    South,
    East,
    West,
}

impl Face {
    pub const ALL: [Face; 6] = [
        Face::Top,
        Face::Bottom,
        Face::North,
        Face::South,
        Face::East,
        Face::West,
    ];

    pub fn normal(self) -> [i32; 3] {
        match self {
            Face::Top => [0, 1, 0],
            Face::Bottom => [0, -1, 0],
            Face::North => [0, 0, -1],
            Face::South => [0, 0, 1],
            Face::East => [1, 0, 0],
            Face::West => [-1, 0, 0],
        }
    }
}

pub type Edits = HashMap<ChunkKey, HashMap<u16, u8>>;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Slide {
    pub position: Vec3,
    pub on_ground: bool,
    pub blocked: [bool; 3],
}
