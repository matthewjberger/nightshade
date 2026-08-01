use nalgebra_glm::Vec3;
pub use nightshade_voxels::data::{CHUNK, FLOOR_Y};

pub const TITLE: &str = "Liquids";
pub const SETTLED: u64 = 600;

pub const ATLAS_TEXTURE: &str = "liquids_atlas";
pub const ATLAS_COLUMNS: u32 = 4;
pub const ATLAS_ROWS: u32 = 2;
pub const TILE_TEXELS: u32 = 16;
pub const ATLAS_CELLS: [f32; 2] = [ATLAS_COLUMNS as f32, ATLAS_ROWS as f32];

pub const SOLID_MATERIAL: &str = "liquids_solid";
pub const WATER_MATERIAL: &str = "liquids_water";
pub const LAVA_MATERIAL: &str = "liquids_lava";

pub const WATER: u8 = 0;
pub const LAVA: u8 = 1;
pub const WATER_REACH: u8 = 4;
pub const LAVA_REACH: u8 = 2;
pub const LAVA_PACE: u8 = 6;

pub const SEA_LEVEL: i32 = 64;
pub const GROUND: f64 = 62.0;
pub const RELIEF: f64 = 9.0;
pub const LAND_SCALE: f64 = 0.02;
pub const HEAT_SCALE: f64 = 0.024;
pub const HEAT_EDGE: f64 = 0.22;
pub const HEAT_POOL: f64 = 0.36;
pub const SOIL_DEPTH: i32 = 3;

pub const STONE_TILE: [u8; 3] = [118, 118, 122];
pub const DIRT_TILE: [u8; 3] = [124, 88, 58];
pub const GRASS_TOP_TILE: [u8; 3] = [92, 158, 62];
pub const GRASS_SIDE_TILE: [u8; 3] = [110, 112, 60];
pub const SAND_TILE: [u8; 3] = [214, 200, 140];
pub const WATER_TILE: [u8; 3] = [200, 220, 245];
pub const LAVA_TILE: [u8; 3] = [235, 120, 40];
pub const OBSIDIAN_TILE: [u8; 3] = [24, 16, 40];
pub const SPECKLE: i32 = 14;

pub const WATER_COLOR: [f32; 4] = [0.28, 0.55, 0.85, 0.72];
pub const WATER_DRIFT: [f32; 2] = [0.08, 0.05];
pub const LAVA_COLOR: [f32; 4] = [1.0, 0.55, 0.2, 1.0];
pub const LAVA_GLOW: [f32; 3] = [0.9, 0.25, 0.03];
pub const LAVA_DRIFT: [f32; 2] = [0.02, 0.012];

pub const EYE: Vec3 = Vec3::new(8.0, 96.0, 56.0);
pub const YAW: f32 = 0.0;
pub const PITCH: f32 = -0.55;
pub const START_HOUR: f32 = 10.5;
