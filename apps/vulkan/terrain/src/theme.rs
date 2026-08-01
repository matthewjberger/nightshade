use nightshade::prelude::Vec3;

pub const TITLE: &str = "Terrain without an end";

pub const SEED: u32 = 0x007E_44A1;

pub const TILE_SPAN: f32 = 96.0;
pub const TILE_CELLS: usize = 96;
pub const MAX_LEVELS: u32 = 288;
pub const VIEW_RADIUS: i32 = 7;
pub const KEEP_MARGIN: i32 = 1;
pub const TILES_IN_FLIGHT: usize = 16;
pub const TILES_TAKEN: usize = 8;

pub const BOULDER_KINDS: usize = 3;
pub const BOULDER_SEGMENTS: u32 = 96;
pub const BOULDERS_PER_TILE: usize = 22;
pub const BOULDER_SMALLEST: f32 = 2.0;
pub const BOULDER_LARGEST: f32 = 8.5;
pub const BOULDER_SINK: f32 = 0.45;
pub const BOULDER_SLOPE_LIMIT: f32 = 0.72;

pub const CONTINENT_FREQUENCY: f64 = 1.0 / 2600.0;
pub const CONTINENT_HEIGHT: f32 = 150.0;
pub const HILL_FREQUENCY: f64 = 1.0 / 420.0;
pub const HILL_HEIGHT: f32 = 52.0;
pub const RIDGE_FREQUENCY: f64 = 1.0 / 620.0;
pub const RIDGE_HEIGHT: f32 = 175.0;
pub const DETAIL_FREQUENCY: f64 = 1.0 / 52.0;
pub const DETAIL_HEIGHT: f32 = 7.5;
pub const ROUGH_FREQUENCY: f64 = 1.0 / 11.0;
pub const ROUGH_HEIGHT: f32 = 1.9;
pub const SEA_LEVEL: f32 = -18.0;

pub const WARP_FREQUENCY: f64 = 1.0 / 46.0;
pub const GRAIN_WARP_FREQUENCY: f64 = 1.0 / 13.0;
pub const OVERHANG_REACH: f32 = 30.0;
pub const OVERHANG_DEPTH: f32 = 21.0;
pub const FIELD_GRAIN_DEPTH: f32 = 2.6;
pub const LEDGE_FREQUENCY: f32 = 0.26;
pub const LEDGE_DEPTH: f32 = 3.2;

pub const LUMP_FREQUENCY: f64 = 1.7;
pub const LUMP_DEPTH: f32 = 0.26;
pub const GRAIN_FREQUENCY: f64 = 5.3;
pub const GRAIN_DEPTH: f32 = 0.085;

pub const GROUND_MATERIAL: &str = "terrain_ground";
pub const GROUND_COLOR: [f32; 4] = [0.30, 0.27, 0.22, 1.0];
pub const GROUND_ROUGHNESS: f32 = 0.92;
pub const ROCK_MATERIAL: &str = "terrain_rock";
pub const ROCK_COLOR: [f32; 4] = [0.25, 0.23, 0.21, 1.0];
pub const ROCK_ROUGHNESS: f32 = 0.84;

pub const CAMERA_START: Vec3 = Vec3::new(0.0, 300.0, 0.0);
pub const CAMERA_YAW: f32 = 0.45;
pub const CAMERA_PITCH: f32 = -0.42;
pub const CAMERA_SPEED: f32 = 96.0;
pub const CAMERA_SPRINT: f32 = 8.0;
pub const CAMERA_NEAR: f32 = 0.35;
pub const CAMERA_FIELD_OF_VIEW: f32 = 58.0;
pub const CAMERA_CLEARANCE: f32 = 8.0;

pub const BLOOM_INTENSITY: f32 = 0.22;
pub const EXPOSURE: f32 = 0.50;

pub const FOG_COLOR: [f32; 3] = [0.40, 0.55, 0.75];
pub const FOG_START_SHARE: f32 = 0.80;
pub const FOG_END_SHARE: f32 = 1.05;
