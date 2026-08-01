use nightshade::prelude::Vec3;

pub const TILE: f32 = 8.0;
pub const CELL_TILES: i32 = 8;
pub const CELL: f32 = TILE * CELL_TILES as f32;
pub const ROAD_LIFT: f32 = -0.16;
pub const GROUND_LIFT: f32 = -0.03;
pub const GROUND_SIZE: f32 = 6000.0;
pub const LANE: f32 = 1.25;
pub const ASPHALT: f32 = 3.2;
pub const PAVEMENT: f32 = 3.2;
pub const SIDEWALK: f32 = 6.4;
pub const KIT_SCALE: f32 = TILE;
pub const TOWER_SCALE: f32 = TILE * 3.0;
pub const CAR_SCALE: f32 = 1.4;
pub const FOLK_SCALE: f32 = 2.6;
pub const FURNITURE_SCALE: f32 = 2.2;
pub const URBAN_SCALE: f32 = 3.0;
pub const BLOCK_SCALE: f32 = 1.25;

pub const ROOM_TILE: f32 = 2.0 * BLOCK_SCALE;
pub const ROOM_FLOOR: f32 = 0.1 * BLOCK_SCALE;
pub const ROOM_TILES: [i32; 2] = [2, 5];
pub const ROOM_SITE: Vec3 = Vec3::new(0.0, -300.0, 0.0);

pub const VIEW_SECTORS: i32 = 8;
pub const VIEW_BEHIND: i32 = 6;
pub const AHEAD_LOAD: f32 = 0.15;
pub const AHEAD_KEEP: f32 = -0.3;
pub const CELLS_IN_FLIGHT: usize = 8;

pub const MODELS_IN_FLIGHT: usize = 16;
pub const SETTLE_FRAMES: u8 = 3;
