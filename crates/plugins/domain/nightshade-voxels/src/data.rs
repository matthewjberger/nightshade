pub const CHUNK: i32 = 16;
pub const CHUNK_BLOCKS: usize = (CHUNK * CHUNK * CHUNK) as usize;
pub const CHUNK_FACES: usize = (CHUNK * CHUNK) as usize;

pub const SLAB: i32 = 2;
pub const SLAB_LEVELS: i32 = 4;

pub const SLAB_SPAN: i32 = SLAB * CHUNK;
pub const SLAB_WIDE: i32 = SLAB_SPAN + 2;
pub const SLAB_HIGH: i32 = CHUNK * SLAB_LEVELS;
pub const SKY_SKIRT: i32 = CHUNK;
pub const SLAB_TALL: i32 = SLAB_HIGH + SKY_SKIRT + 1;
pub const SLAB_CELLS: usize = (SLAB_WIDE * SLAB_TALL * SLAB_WIDE) as usize;
pub const SLAB_COLUMNS: i32 = SLAB + 2;

pub const WORLD_Y: i32 = 256;
pub const FLOOR_Y: i32 = -64;
pub const WORLD_TALL: i32 = WORLD_Y - FLOOR_Y;
pub const FLOOR_LEVEL: i32 = FLOOR_Y / CHUNK;
pub const TOP_LEVEL: i32 = WORLD_Y / CHUNK;
pub const COLUMN_CHUNKS: i32 = TOP_LEVEL - FLOOR_LEVEL;

pub const SURFACE_NONE: i16 = (FLOOR_Y - 1) as i16;

pub const SKY_LEVELS: u8 = 15;
pub const CAVE_FLOOR: f32 = 0.04;
pub const AMBIENT_LEVELS: [f32; 4] = [0.5, 0.68, 0.85, 1.0];

pub const LEVEL_RANK: i32 = 12;
pub const MESH_BATCH: usize = 96;
pub const SLABS_IN_FLIGHT: usize = 12;
pub const MASON_LIMIT: usize = 8;
pub const PLACE_LIMIT: usize = 2;
pub const DIGGER_LIMIT: usize = 6;
pub const DIGGER_SHARE: usize = 3;
pub const CREW_SHARES: usize = 5;
pub const COLUMNS_IN_FLIGHT: usize = 32;
pub const SETTLE_LIMIT: usize = 2;
pub const UNLOAD_STEP: usize = 2;
pub const SHADOW_STEP: usize = 2;
pub const FALLS_PER_FRAME: usize = 512;
pub const FALLING_LIMIT: usize = 4096;
pub const FALL_ACCELERATION: f32 = 24.0;
pub const FALL_TERMINAL: f32 = 60.0;
pub const FALL_STEP_LIMIT: f32 = 0.05;
pub const FALL_SEARCH: i32 = 8;
pub const LIQUID_STEPS_PER_FRAME: usize = 48;

pub const VIEW_RADIUS: i32 = 20;
pub const KEEP_RADIUS: i32 = VIEW_RADIUS + 2;
pub const SHADOW_RADIUS: i32 = 2;
pub const SHADOW_LEVELS: i32 = 0;

pub const CAVE_SIGHT: i32 = 5;
pub const CAVE_SPAN: i32 = 2 * CAVE_SIGHT + 1;
pub const CAVE_CELLS: usize = (CAVE_SPAN * CAVE_SPAN * CAVE_SPAN) as usize;
pub const SIGHT_SECONDS: f32 = 0.2;
pub const VIEW_SECTORS: f32 = 16.0;

pub const MESH_PREFIX: &str = "voxel_slab";
pub const VIEW_BEHIND: i32 = 6;

pub const CROSS_OFFSET: f32 = 0.0015;
