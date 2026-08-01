use nalgebra_glm::{Vec3, Vec4};
pub use nightshade_voxels::data::{
    CHUNK, COLUMN_CHUNKS, FLOOR_LEVEL, FLOOR_Y, WORLD_TALL, WORLD_Y,
};

pub const TILE_TEXELS: u32 = 16;
pub const TORCH_WALL_RISE: f32 = 0.22;
pub const TORCH_WALL_SHIFT: f32 = 0.42;
pub const TORCH_WALL_TILT: f32 = 0.2;

pub const TITLE: &str = "Voxels";

pub const ATLAS_BYTES: &[u8] = include_bytes!("../assets/blocks_atlas.png");
pub const KENNEY_ATLAS_BYTES: &[u8] = include_bytes!("../assets/kenney_atlas.png");
pub const ATLAS_TEXTURE: &str = "voxels_block_atlas";
pub const ATLAS_COLUMNS: u32 = 32;
pub const ATLAS_ROWS: u32 = 16;
pub const ATLAS_CELLS: [f32; 2] = [ATLAS_COLUMNS as f32, ATLAS_ROWS as f32];

pub const SOLID_MATERIAL: &str = "voxels_solid";
pub const CUTOUT_MATERIAL: &str = "voxels_cutout";
pub const TRANSPARENT_MATERIAL: &str = "voxels_transparent";
pub const GLASS_ROUGHNESS: f32 = 0.08;
pub const WATER_MATERIAL: &str = "voxels_water";
pub const WATER: u8 = 0;
pub const WATER_RUN: u8 = 3;
pub const RIPPLE_TEXTURE: &str = "voxels_ripples";
pub const RIPPLE_TEXELS: u32 = 64;
pub const RIPPLE_HEIGHT: f32 = 0.09;
pub const RIPPLE_SCALE: f32 = 3.0;
pub const RIPPLE_DRIFT: [f32; 2] = [0.05, 0.031];
pub const WATER_DRIFT: [f32; 2] = [0.09, 0.055];
pub const RIPPLE_STRENGTH: f32 = 0.3;
pub const WATER_ROUGHNESS: f32 = 0.34;
pub const FLAT_MATERIAL: &str = "voxels_flat";
pub const BOAT_MESH: &str = "voxels_boat";
pub const BOAT_SPEED: f32 = 8.0;
pub const BOAT_EASE: f32 = 3.5;
pub const BOAT_FLOAT: f32 = 0.22;
pub const BOAT_REACH: f32 = 2.2;

pub const CRACK_MESH: &str = "voxels_crack";
pub const CRACK_INFLATE: f32 = 0.004;
pub const SWING_SECONDS: f32 = 0.28;
pub const TOOL_SHEET_WIDTH: u32 = 512;
pub const TOOL_SHEET_HEIGHT: u32 = 128;

pub const GRASS_TINT: [f32; 3] = [0.55, 0.85, 0.35];
pub const LEAF_TINT: [f32; 3] = [0.35, 0.72, 0.25];

pub const HEIGHT_SCALE: f32 = 3.0;
pub const SEA_LEVEL: i32 = 66;
pub const HEART_COUNT: usize = 10;
pub const HEART_SIZE: f32 = 15.0;
pub const HEART_GAP: f32 = 3.0;
pub const VITAL_BOTTOM: f32 = 104.0;
pub const FULL_HEALTH: f32 = 20.0;
pub const FULL_FOOD: f32 = 20.0;
pub const FOOD_BURN: f32 = 0.055;
pub const FOOD_HEAL: f32 = 14.0;
pub const HEAL_RATE: f32 = 0.5;
pub const STARVE_RATE: f32 = 0.35;
pub const DROWN_SECONDS: f32 = 12.0;
pub const DROWN_RATE: f32 = 2.0;
pub const FALL_FLOOR: f32 = 9.0;
pub const FALL_HURT: f32 = 1.6;
pub const HURT_SHAKE: f32 = 0.45;
pub const CARROT_FOOD: f32 = 4.0;
pub const WHEAT_FOOD: f32 = 3.0;
pub const HEART_FULL: Vec4 = Vec4::new(0.88, 0.22, 0.24, 1.0);
pub const HEART_EMPTY: Vec4 = Vec4::new(0.18, 0.12, 0.14, 0.75);
pub const FOOD_FULL: Vec4 = Vec4::new(0.92, 0.66, 0.24, 1.0);
pub const FOOD_EMPTY: Vec4 = Vec4::new(0.18, 0.15, 0.10, 0.75);
pub const ROCK_RELIEF: f32 = 16.0;
pub const ROCK_FLOOR: f32 = 0.18;
pub const ROCK_RIDGE: f32 = 0.62;
pub const ROCK_CRAG: f32 = 0.26;
pub const SHAPE_UP: f32 = ROCK_RIDGE * 0.5 + ROCK_CRAG;
pub const SHAPE_DOWN: f32 = ROCK_RIDGE * 1.5 + ROCK_CRAG;
pub const LATTICE_STEP: i32 = 4;
pub const LATTICE_NODES: i32 = CHUNK / LATTICE_STEP + 3;
pub const LATTICE_LEVELS: i32 = WORLD_TALL / LATTICE_STEP + 1;
pub const APRON: f32 = 6.0;
pub const PAD_STEP: i32 = 6;
pub const CHIP_REACH: f32 = 1.2;
pub const CHIP_ROCK: f32 = 1.5;
pub const CHIP_SHELL: f32 = 2.0;
pub const CLUMP_RUN: i32 = 2;
pub const CLUMP_GAP: i32 = 3;
pub const CRAG_SCALE: f64 = 13.0;
pub const SHELF_STEP: f32 = 4.0;
pub const SHELF_BITE: f32 = 0.55;

pub const CAVERN_EDGE: f64 = 0.42;
pub const CAVERN_TOP: i32 = 90;
pub const RAVINE_WIDTH: f64 = 0.012;
pub const RAVINE_FLOOR: i32 = -36;
pub const FLOOD_LEVEL: i32 = 39;
pub const SEED: u32 = 1337;

pub const GLOW_RADIUS: i32 = 4;
pub const SPAWN_RADIUS: i32 = 2;
pub const SPAWN_SEARCH: i32 = 20;
pub const SPAWN_SEARCH_WIDE: i32 = 140;

pub const REGION: i32 = 224;
pub const DUNGEON_CHANCE: u32 = 70;
pub const DUNGEON_ROOMS: i32 = 3;
pub const DUNGEON_ROOM: i32 = 9;
pub const DUNGEON_TALL: i32 = 5;
pub const DUNGEON_LOW: i32 = -50;
pub const DUNGEON_HIGH: i32 = 20;
pub const DUNGEON_PRIZES: usize = 5;
pub const DUNGEON_TUNNEL: i32 = 8;

pub const SAVE_VERSION: u32 = 3;
pub const SAVE_FOLDER: &str = "nightshade-voxels";
pub const SAVE_NAME: &str = "world";
pub const SAVE_SLOT: u32 = 0;
pub const SAVE_EVERY: f32 = 20.0;
pub const VILLAGE_CHANCE: u32 = 60;
pub const VILLAGE_REACH: i32 = 42;
pub const TREE_CELL: i32 = 5;

pub const PLAYER_HALF_WIDTH: f32 = 0.3;
pub const PLAYER_HEIGHT: f32 = 1.8;
pub const EYE_HEIGHT: f32 = 1.62;
pub const WALK_SPEED: f32 = 4.3;
pub const SPRINT_SPEED: f32 = 6.5;
pub const FLY_SPEED: f32 = 12.0;
pub const WATCH_SPEED: f32 = 22.0;
pub const WATCH_SPEED_MIN: f32 = 6.0;
pub const WATCH_SPEED_MAX: f32 = 90.0;
pub const WATCH_SPEED_STEP: f32 = 4.0;
pub const WATCH_PICK_REACH: f32 = 46.0;
pub const WATCH_PICK_COSINE: f32 = 0.985;
pub const WATCH_EYE_LIFT: f32 = 0.55;
pub const SWIM_SPEED: f32 = 2.6;
pub const JUMP_SPEED: f32 = 8.6;
pub const GRAVITY: f32 = 28.0;
pub const WATER_GRAVITY: f32 = 6.0;
pub const WATER_REACH: i32 = 4;
pub const DRY_GROW_MULTIPLIER: f32 = 0.25;
pub const MOUSE_SENSITIVITY: f32 = 0.0022;
pub const PAD_LOOK_RATE: f32 = 2.8;
pub const REACH: f32 = 5.5;
pub const PICKUP_RADIUS: f32 = 1.4;
pub const PICKUP_DELAY: f32 = 0.6;
pub const ITEM_LIFETIME: f32 = 300.0;

pub const START_HOUR: f32 = 7.5;
pub const DAY_SPEED: f32 = 0.05;
pub const DAWN_HOUR: f32 = 5.5;
pub const DUSK_HOUR: f32 = 18.5;
pub const SKY_DAY: Vec3 = Vec3::new(0.62, 0.76, 0.95);
pub const SKY_DAWN: Vec3 = Vec3::new(0.85, 0.62, 0.5);
pub const SKY_DUSK: Vec3 = Vec3::new(0.72, 0.45, 0.42);
pub const SKY_NIGHT: Vec3 = Vec3::new(0.06, 0.08, 0.16);
pub const FOG_START: f32 = 243.0;
pub const FOG_END: f32 = 315.0;
pub const FOG_HEIGHT: f32 = 90.0;
pub const FOG_FALLOFF: f32 = 34.0;
pub const BLOOM_DAY: f32 = 0.1;
pub const BLOOM_NIGHT: f32 = 0.24;
pub const BLOOM_FLASH: f32 = 0.85;
pub const SKY_OVERCAST: Vec3 = Vec3::new(0.44, 0.46, 0.5);
pub const RAIN_FOG_START: f32 = 120.0;
pub const RAIN_FOG_END: f32 = 260.0;

pub const WEATHER_FIRST: f32 = 45.0;
pub const WEATHER_SHORTEST: f32 = 110.0;
pub const WEATHER_LONGEST: f32 = 280.0;
pub const WEATHER_FADE: f32 = 5.0;
pub const CLEAR_SHARE: f32 = 0.45;
pub const CLOUDY_SHARE: f32 = 0.25;
pub const RAIN_SHARE: f32 = 0.2;
pub const PARCHED_CLEAR: f32 = 0.8;
pub const RAIN_HEIGHT: f32 = 12.0;
pub const RAIN_REACH: Vec3 = Vec3::new(15.0, 0.4, 15.0);
pub const RAIN_RATE: f32 = 520.0;
pub const SNOW_RATE: f32 = 180.0;
pub const LIGHTNING_GAP_MIN: f32 = 5.0;
pub const LIGHTNING_GAP_MAX: f32 = 24.0;
pub const FLASH_SECONDS: f32 = 0.25;
pub const SUN_SHADOW_SOFTNESS: f32 = 1.4;

pub const CAMERA_FOV_DEGREES: f32 = 72.0;
pub const CAMERA_NEAR: f32 = 0.05;
pub const CAMERA_FAR: f32 = 1200.0;

pub const HOTBAR_SLOTS: usize = 9;
pub const INVENTORY_ROWS: usize = 4;
pub const INVENTORY_SLOTS: usize = HOTBAR_SLOTS * INVENTORY_ROWS;
pub const CHEST_SLOTS: usize = 18;
pub const CHEST_ROW: usize = 9;
pub const GLASS_CELL: (u32, u32) = (9, 2);
pub const CHEST_TOP_CELL: (u32, u32) = (24, 0);
pub const CHEST_SIDE_CELL: (u32, u32) = (25, 0);
pub const SLOT_SIZE: f32 = 52.0;
pub const SLOT_GAP: f32 = 4.0;
pub const ICON_SIZE: f32 = 34.0;
pub const HOTBAR_BOTTOM_MARGIN: f32 = 18.0;

pub const SLOT_COLOR: Vec4 = Vec4::new(0.08, 0.08, 0.1, 0.72);
pub const SLOT_BORDER: Vec4 = Vec4::new(0.45, 0.45, 0.5, 0.9);
pub const SLOT_SELECTED_COLOR: Vec4 = Vec4::new(0.25, 0.25, 0.28, 0.9);
pub const SLOT_SELECTED_BORDER: Vec4 = Vec4::new(1.0, 1.0, 1.0, 1.0);
pub const TEXT_COLOR: Vec4 = Vec4::new(1.0, 1.0, 1.0, 1.0);
pub const CROSSHAIR_COLOR: Vec4 = Vec4::new(1.0, 1.0, 1.0, 0.85);
pub const OUTLINE_COLOR: Vec4 = Vec4::new(0.05, 0.05, 0.05, 1.0);
pub const WATER_COLOR: [f32; 4] = [0.30, 0.62, 0.78, 1.0];

pub const SETTLED: u64 = 120;
pub const ITEM_DEPTH: f32 = 2.0;

pub const SHEET_BYTES: &[u8] = include_bytes!("../assets/tiny_farm/tilemap_packed.png");
pub const SPRITE_ATLAS: &str = "voxels_sprites";
pub const SPRITE_SHEET: &str = "tiny_farm";
pub const SPRITE_ATLAS_SIZE: u32 = 512;
pub const SHEET_COLUMNS: u32 = 12;
pub const SHEET_ROWS: u32 = 11;
pub const SHEET_TILES: usize = (SHEET_COLUMNS * SHEET_ROWS) as usize;
pub const SPRITE_LAYER: i32 = 1;

pub const SHOPKEEPER: u32 = 109;
pub const VILLAGER: u32 = 108;
pub const COW: u32 = 121;
pub const CHICKEN: u32 = 122;
pub const SHEEP: u32 = 120;
pub const GOAT: u32 = 123;
pub const VILLAGER_TINT: [f32; 4] = [0.82, 0.92, 1.0, 1.0];
pub const LABEL_HEIGHT: f32 = 1.45;
pub const STEP_SMOOTHING: f32 = 12.0;

pub const DUNGEON_SHEET_BYTES: &[u8] = include_bytes!("../assets/tiny_dungeon/tilemap_packed.png");
pub const DUNGEON_SHEET: &str = "tiny_dungeon";
pub const SKELETON_TILE: u32 = 86;
pub const SLIME_TILE: u32 = 108;
pub const BAT_TILE: u32 = 120;
pub const GHOST_TILE: u32 = 121;
pub const SPIDER_TILE: u32 = 122;

pub const MONSTER_CAP: usize = 16;
pub const MONSTER_WAIT: f32 = 1.1;
pub const MONSTER_NEAR: i32 = 9;
pub const MONSTER_FAR: i32 = 26;
pub const MONSTER_ROOF: i32 = 3;
pub const MONSTER_FORGET: f32 = 60.0;
pub const MONSTER_CHASE: f32 = 22.0;
pub const MONSTER_TOUCH: f32 = 1.15;
pub const MONSTER_BITE_WAIT: f32 = 1.1;
pub const MONSTER_HURT_FLASH: f32 = 0.3;
pub const MONSTER_HURT_TINT: [f32; 4] = [1.0, 0.45, 0.4, 1.0];
pub const MONSTER_FLOAT: f32 = 1.1;
pub const MONSTER_BOB: f32 = 0.14;
pub const MONSTER_BOB_RATE: f32 = 3.4;
pub const MONSTER_FALL: i32 = 3;
pub const MONSTER_RISE: i32 = 2;
pub const SWING_REACH: f32 = 3.4;
pub const SWING_ARC: f32 = 0.55;
pub const SWORD_HIT: f32 = 3.0;
pub const FIST_HIT: f32 = 1.0;

pub const VILLAGE_NAME: &str = "Willowbrook";
pub const STARTING_COINS: u32 = 20;
pub const TALK_DISTANCE: f32 = 4.5;
pub const TALK_COSINE: f32 = 0.94;
pub const TYPEWRITER_RATE: f32 = 45.0;
pub const TALK_HEAD: f32 = 1.05;
pub const TALK_BACK: f32 = 1.55;
pub const TALK_SIDE: f32 = 1.15;
pub const TALK_LIFT: f32 = 0.45;
pub const TALK_FOCUS: f32 = 0.68;
pub const TALK_BLEND_RATE: f32 = 3.2;
pub const TALK_BOUNCE: f32 = 0.075;
pub const TALK_BOUNCE_RATE: f32 = 7.5;
pub const TALK_SWAY: f32 = 0.085;
pub const TALK_SWAY_RATE: f32 = 2.6;
pub const TALK_CLEARANCE: f32 = 0.35;
pub const TOAST_SECONDS: f32 = 2.5;

pub const FIREWORK_FUSE: f32 = 1.5;
pub const FIREWORK_SPEED: f32 = 17.0;
pub const FIREWORK_PARTICLES: u32 = 220;
pub const FIREWORK_SPARK_SIZE: f32 = 0.12;
pub const FIREWORK_SCALE: f32 = 0.4;
pub const FIREWORK_COLORS: [Vec3; 5] = [
    Vec3::new(1.0, 0.35, 0.3),
    Vec3::new(0.4, 0.7, 1.0),
    Vec3::new(1.0, 0.85, 0.35),
    Vec3::new(0.5, 1.0, 0.5),
    Vec3::new(1.0, 0.5, 0.9),
];

pub const CAMPFIRE_COLOR: Vec3 = Vec3::new(1.0, 0.68, 0.36);
pub const CAMPFIRE_STRENGTH: f32 = 18.0;
pub const CAMPFIRE_RANGE: f32 = 11.0;

pub const TORCH_COLOR: Vec3 = Vec3::new(1.0, 0.72, 0.42);
pub const TORCH_STRENGTH: f32 = 9.0;
pub const TORCH_RANGE: f32 = 13.0;
pub const TORCH_FLAME_HEIGHT: f32 = 0.66;

pub const PANEL_COLOR: Vec4 = Vec4::new(0.07, 0.06, 0.09, 0.96);
pub const PANEL_BORDER: Vec4 = Vec4::new(0.85, 0.7, 0.45, 0.9);
pub const TITLE_COLOR: Vec4 = Vec4::new(0.16, 0.13, 0.09, 1.0);
pub const FOOTER_COLOR: Vec4 = Vec4::new(0.11, 0.1, 0.13, 1.0);
pub const ROW_COLOR: Vec4 = Vec4::new(0.15, 0.14, 0.18, 0.75);
pub const ROW_ALT_COLOR: Vec4 = Vec4::new(0.11, 0.1, 0.14, 0.75);
pub const CELL_COLOR: Vec4 = Vec4::new(0.13, 0.12, 0.16, 0.95);
pub const CELL_BORDER: Vec4 = Vec4::new(0.42, 0.4, 0.46, 0.95);
pub const HELD_COLOR: Vec4 = Vec4::new(0.28, 0.24, 0.14, 1.0);
pub const HELD_BORDER: Vec4 = Vec4::new(1.0, 0.84, 0.45, 1.0);
pub const ACCENT_COLOR: Vec4 = Vec4::new(1.0, 0.82, 0.45, 1.0);
pub const DIM_TEXT_COLOR: Vec4 = Vec4::new(0.7, 0.66, 0.6, 1.0);
pub const READY_COLOR: Vec4 = Vec4::new(0.62, 0.9, 0.55, 1.0);
pub const SHORT_COLOR: Vec4 = Vec4::new(0.85, 0.45, 0.42, 1.0);

pub const HEADING_SIZE: f32 = 24.0;
pub const BODY_SIZE: f32 = 18.0;
pub const SMALL_SIZE: f32 = 15.0;
pub const MENU_WIDTH: f32 = 660.0;
pub const SETTINGS_HEIGHT: f32 = 397.0;
pub const TITLE_MENU_HEIGHT: f32 = 300.0;
pub const NEW_WORLD_HEIGHT: f32 = 540.0;
pub const FLAT_HEIGHT: i32 = SEA_LEVEL + 4;
pub const ISLAND_DROP: f32 = 7.0;
pub const ISLAND_RELIEF: f32 = 1.5;
pub const EXTREME_RELIEF: f32 = 2.6;
pub const PEAKS_RELIEF: f32 = 2.3;
pub const POCKET_HEIGHT: f32 = 560.0;
pub const CHEST_HEIGHT: f32 = 640.0;
pub const SHOP_HEIGHT: f32 = 560.0;
pub const CRAFTING_HEIGHT: f32 = 560.0;
pub const DIALOGUE_WIDTH: f32 = 760.0;
pub const DIALOGUE_HEIGHT: f32 = 130.0;

pub const TITLE_HEIGHT: f32 = 48.0;
pub const FOOTER_HEIGHT: f32 = 58.0;
pub const BODY_PADDING: f32 = 16.0;
pub const ROW_HEIGHT: f32 = 38.0;
pub const ROW_GAP: f32 = 4.0;
pub const ROW_PADDING: f32 = 8.0;
pub const ROW_SPACING: f32 = 8.0;
pub const ROW_ICON: f32 = 26.0;
pub const ROW_WIDTH: f32 = MENU_WIDTH - 60.0;
pub const NAME_WIDTH: f32 = 214.0;
pub const PRICE_WIDTH: f32 = 116.0;
pub const HAVE_WIDTH: f32 = 104.0;
pub const NEEDS_WIDTH: f32 = 224.0;
pub const ACTION_WIDTH: f32 = 88.0;
pub const ACTION_HEIGHT: f32 = 30.0;
pub const CELL_SIZE: f32 = 58.0;
pub const CELL_ICON: f32 = 36.0;
pub const CELL_GAP: f32 = 8.0;
pub const COUNT_SIZE: f32 = 14.0;
pub const COINS_WIDTH: f32 = 170.0;
pub const COINS_CARD_WIDTH: f32 = 150.0;
pub const RATE_CARD_WIDTH: f32 = 132.0;
pub const SETTING_NAME_WIDTH: f32 = 396.0;
pub const SEED_WIDTH: f32 = 96.0;
pub const TITLE_TURN: f32 = 0.06;
pub const TITLE_PITCH: f32 = -0.12;
pub const RATE_GOOD_COLOR: Vec4 = Vec4::new(0.4, 1.0, 0.5, 1.0);
pub const RATE_FAIR_COLOR: Vec4 = Vec4::new(1.0, 0.85, 0.3, 1.0);
pub const RATE_POOR_COLOR: Vec4 = Vec4::new(1.0, 0.36, 0.3, 1.0);
pub const RATE_GOOD: f32 = 110.0;
pub const RATE_FAIR: f32 = 55.0;
pub const STATUS_WIDTH: f32 = 460.0;
pub const STATUS_HEIGHT: f32 = 34.0;
