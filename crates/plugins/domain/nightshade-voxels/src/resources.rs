use crate::contract::{Blocks, ChunkKey, ChunkLayer, ColumnKey, Edits, SlabKey};
use crate::data;
use nalgebra_glm::{IVec3, Vec3};
use nightshade::ecs::Entity;
use nightshade_render::data::geometry::Mesh;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Default)]
pub enum ChunkStage {
    #[default]
    Empty,
    Terrain,
    Decorated,
    Meshed,
}

#[derive(Clone)]
pub struct Chunk {
    pub blocks: Arc<Vec<u8>>,
    pub stage: ChunkStage,
    pub openings: u64,
    pub solid: bool,
}

impl Default for Chunk {
    fn default() -> Self {
        Self {
            blocks: Arc::new(vec![0; data::CHUNK_BLOCKS]),
            stage: ChunkStage::Empty,
            openings: 0,
            solid: false,
        }
    }
}

pub struct VoxelWorld {
    pub blocks: Arc<Blocks>,
    pub chunks: HashMap<ChunkKey, Chunk>,
    pub uniform: HashMap<u8, Arc<Vec<u8>>>,
    pub dirty: HashSet<SlabKey>,
    pub updates: Vec<IVec3>,
    pub wet: Vec<IVec3>,
    pub ticks: u64,
    pub met: Vec<(IVec3, IVec3)>,
    pub markers: HashMap<ChunkKey, HashMap<[i32; 3], u8>>,
    pub surface: HashMap<ColumnKey, Arc<Vec<i16>>>,
    pub tops: HashMap<ColumnKey, i32>,
    pub edited: Edits,
    pub edits: u64,
    pub sight: u64,
    pub fallen: Vec<(IVec3, u8)>,
}

impl Default for VoxelWorld {
    fn default() -> Self {
        Self {
            blocks: Arc::new(Blocks::default()),
            chunks: HashMap::new(),
            uniform: HashMap::new(),
            dirty: HashSet::new(),
            updates: Vec::new(),
            wet: Vec::new(),
            ticks: 0,
            met: Vec::new(),
            markers: HashMap::new(),
            surface: HashMap::new(),
            tops: HashMap::new(),
            edited: HashMap::new(),
            edits: 0,
            sight: 0,
            fallen: Vec::new(),
        }
    }
}

impl VoxelWorld {
    pub fn same_blocks(&self) -> Self {
        Self {
            blocks: self.blocks.clone(),
            ..Default::default()
        }
    }
}

#[derive(Default)]
pub struct Meshes {
    pub entities: HashMap<(SlabKey, ChunkLayer), Entity>,
    pub casting: HashSet<SlabKey>,
    pub center: Option<SlabKey>,
    pub ring: VecDeque<(SlabKey, bool)>,
}

pub struct Fall {
    pub block: u8,
    pub position: Vec3,
    pub speed: f32,
    pub entity: Entity,
}

#[derive(Default)]
pub struct Falling {
    pub blocks: Vec<Fall>,
    pub meshed: HashSet<u8>,
}

#[derive(Default)]
pub struct VoxelViewer {
    pub eye: Vec3,
    pub forward: Vec3,
    pub whole: bool,
    pub shown: HashSet<SlabKey>,
}

pub struct VoxelSettings {
    pub solid_material: String,
    pub cutout_material: String,
    pub transparent_material: String,
    pub floor_block: u8,
    pub block_size: f32,
    pub view_radius: i32,
    pub keep_radius: i32,
    pub shadow_radius: i32,
    pub shadow_levels: i32,
}

impl Default for VoxelSettings {
    fn default() -> Self {
        Self {
            solid_material: String::new(),
            cutout_material: String::new(),
            transparent_material: String::new(),
            floor_block: 0,
            block_size: 1.0,
            view_radius: data::VIEW_RADIUS,
            keep_radius: data::KEEP_RADIUS,
            shadow_radius: data::SHADOW_RADIUS,
            shadow_levels: data::SHADOW_LEVELS,
        }
    }
}

#[derive(Default)]
pub struct Streaming {
    pub center: Option<ColumnKey>,
    pub facing: Option<i32>,
    pub pending: Vec<ColumnKey>,
    pub leaving: Vec<ColumnKey>,
}

#[derive(Default)]
pub struct Sight {
    pub center: Option<ChunkKey>,
    pub stamp: u64,
    pub whole: bool,
    pub wait: f32,
    pub entered: Vec<u8>,
}

pub trait Dig: Send + Sync + 'static {
    type Orders: Send + Sync + 'static;
    type Order: Send + 'static;
    type Tools: Default + Send + Sync + 'static;
    type Report: Send + 'static;

    fn order(orders: &mut Self::Orders, column: ColumnKey) -> Self::Order;

    fn dig(
        order: Self::Order,
        tools: &mut Self::Tools,
        world: &mut VoxelWorld,
        column: ColumnKey,
    ) -> Self::Report;

    fn settle(
        orders: &mut Self::Orders,
        world: &mut VoxelWorld,
        column: ColumnKey,
        report: Self::Report,
    ) -> Vec<ColumnKey>;
}

pub struct ColumnJob<D: Dig> {
    pub column: ColumnKey,
    pub order: D::Order,
    pub edited: Edits,
    pub blocks: Arc<Blocks>,
}

pub struct ColumnWork<D: Dig> {
    pub column: ColumnKey,
    pub chunks: HashMap<ChunkKey, Chunk>,
    pub surface: Arc<Vec<i16>>,
    pub markers: HashMap<ChunkKey, HashMap<[i32; 3], u8>>,
    pub report: D::Report,
}

pub struct Ground {
    pub key: SlabKey,
    pub chunks: Vec<Option<Chunk>>,
    pub heights: Vec<Option<Arc<Vec<i16>>>>,
    pub blocks: Arc<Blocks>,
    pub floor_block: u8,
}

pub struct SlabWork {
    pub key: SlabKey,
    pub meshes: Vec<(ChunkLayer, Mesh)>,
    pub openings: Vec<(ChunkKey, u64)>,
}

pub struct Neighborhood {
    pub origin: IVec3,
    pub blocks: Vec<u8>,
    pub tops: Vec<i32>,
    pub sky: Vec<u8>,
    pub table: Arc<Blocks>,
}

impl Neighborhood {
    pub fn cell(&self, position: IVec3) -> Option<usize> {
        let local = position - self.origin;
        let wide = data::SLAB_WIDE;
        if !(0..wide).contains(&local.x)
            || !(0..data::SLAB_TALL).contains(&local.y)
            || !(0..wide).contains(&local.z)
        {
            return None;
        }
        Some(((local.y * wide + local.z) * wide + local.x) as usize)
    }

    pub fn at(&self, position: IVec3) -> u8 {
        self.cell(position)
            .map(|cell| self.blocks[cell])
            .unwrap_or(crate::contract::AIR)
    }

    pub fn sky_at(&self, position: IVec3) -> u8 {
        self.cell(position).map(|cell| self.sky[cell]).unwrap_or(0)
    }
}
