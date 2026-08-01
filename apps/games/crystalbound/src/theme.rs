use nalgebra_glm::{Vec2, Vec4};

pub const TITLE: &str = "Crystalbound";

pub const ATLAS: &str = "crystalbound";
pub const ATLAS_SIZE: u32 = 1024;

pub const TOWN_SHEET: &str = "tiny_town";
pub const TOWN_BYTES: &[u8] = include_bytes!("../assets/tiny_town/tilemap_packed.png");
pub const DUNGEON_SHEET: &str = "tiny_dungeon";
pub const DUNGEON_BYTES: &[u8] = include_bytes!("../assets/tiny_dungeon/tilemap_packed.png");
pub const SHEET_COLUMNS: u32 = 12;
pub const SHEET_ROWS: u32 = 11;
pub const TILE_TEXELS: u32 = 16;

pub const WATER_SHEET: &str = "water";
pub const GEM_SHEET: &str = "gem";
pub const SHADE_SHEET: &str = "shade";

pub const TILE: f32 = 1.0;
pub const TILE_SIZE: Vec2 = Vec2::new(TILE, TILE);
pub const PIXELS_PER_UNIT: f32 = 48.0;

pub const LAYER_GROUND: i32 = 0;
pub const LAYER_WATER: i32 = 1;
pub const LAYER_DECOR: i32 = 2;
pub const LAYER_SHADE: i32 = 3;
pub const LAYER_ACTOR: i32 = 4;
pub const LAYER_BLADE: i32 = 5;

pub const GRASS: u32 = 0;
pub const FLOWERS: u32 = 2;
pub const DIRT: u32 = 25;
pub const MOUNTAIN: u32 = 121;
pub const TREE_TOP: u32 = 4;
pub const TREE_MIDDLE: u32 = 16;
pub const TREE_BASE: u32 = 28;
pub const CAVE_TOP_LEFT: u32 = 111;
pub const CAVE_TOP_RIGHT: u32 = 112;
pub const CAVE_LEFT: u32 = 123;
pub const CAVE_RIGHT: u32 = 124;
pub const WALL_PLAIN: u32 = 73;
pub const WALL_WINDOW: u32 = 84;
pub const WALL_DOOR: u32 = 86;
pub const ROOF_RED: [u32; 3] = [52, 53, 54];
pub const ROOF_BLUE: [u32; 3] = [48, 49, 50];
pub const FENCE: u32 = 81;

pub const VAULT_FLOOR: [u32; 3] = [48, 49, 50];
pub const VAULT_WALL: u32 = 57;
pub const VAULT_STAIRS: u32 = 38;
pub const CHEST_SHUT: u32 = 89;
pub const CHEST_OPEN: u32 = 91;

pub const HERO_TILES: [u32; 4] = [96, 84, 98, 100];
pub const SLIME_TILE: u32 = 108;
pub const WOLF_TILE: u32 = 111;
pub const IMP_TILE: u32 = 110;
pub const BAT_TILE: u32 = 120;
pub const SPIDER_TILE: u32 = 122;
pub const GHOST_TILE: u32 = 121;
pub const BLADE_TILE: u32 = 104;

pub const WATER_COLOR: [u8; 4] = [60, 116, 190, 255];
pub const WATER_GLINT: [u8; 4] = [96, 158, 226, 255];
pub const GEM_CORE: [u8; 4] = [126, 232, 244, 255];
pub const GEM_EDGE: [u8; 4] = [54, 150, 186, 255];
pub const GEM_SHINE: [u8; 4] = [236, 254, 255, 255];
pub const SHADE_COLOR: [u8; 4] = [0, 0, 0, 132];

pub const VAULT_TINT: [f32; 4] = [0.62, 0.60, 0.78, 1.0];
pub const ACTOR_TINT: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
pub const HURT_TINT: [f32; 4] = [1.0, 0.45, 0.45, 1.0];

pub const WALK_SPEED: f32 = 5.0;
pub const ROAMER_SPEED: f32 = 2.1;
pub const ROAMER_SIGHT: f32 = 6.0;
pub const BODY_RADIUS: f32 = 0.32;
pub const BLADE_REACH: f32 = 0.9;
pub const BLADE_LIFE: f32 = 0.18;
pub const BLADE_SPAN: f32 = 0.8;
pub const SWING_REST: f32 = 0.34;
pub const TOUCH_RANGE: f32 = 0.7;
pub const TOUCH_REST: f32 = 1.0;
pub const REACH: f32 = 1.1;

pub const CAMERA_FOLLOW: f32 = 8.0;
pub const CAMERA_DEPTH: f32 = 20.0;

pub const STAGE_ORIGIN: Vec2 = Vec2::new(0.0, 400.0);
pub const STAGE_FOE_X: f32 = -4.4;
pub const STAGE_HERO_X: f32 = 3.2;
pub const STAGE_ROW: f32 = 1.6;

pub const ENCOUNTER_DISTANCE: f32 = 13.0;
pub const ENCOUNTER_SPREAD: f32 = 9.0;
pub const INN_PRICE: u32 = 10;
pub const START_GOLD: u32 = 60;
pub const START_POTIONS: u32 = 2;
pub const LEVEL_STEP: u32 = 24;
pub const MAX_UPGRADE: u32 = 3;

pub const BEAT_INTRO: f32 = 0.9;
pub const BEAT_ACTION: f32 = 0.7;
pub const BEAT_CLOSE: f32 = 1.4;
pub const TOAST_LIFE: f32 = 2.4;

pub const PANEL_BG: Vec4 = Vec4::new(0.05, 0.05, 0.09, 0.88);
pub const PANEL_BORDER: Vec4 = Vec4::new(0.55, 0.72, 0.92, 0.35);
pub const TEXT_BRIGHT: Vec4 = Vec4::new(0.96, 0.97, 1.0, 1.0);
pub const TEXT_DIM: Vec4 = Vec4::new(0.70, 0.74, 0.86, 1.0);
pub const TEXT_GOLD: Vec4 = Vec4::new(1.0, 0.84, 0.36, 1.0);
pub const TEXT_HURT: Vec4 = Vec4::new(1.0, 0.44, 0.44, 1.0);

pub const TITLE_SIZE: f32 = 40.0;
pub const HEADING_SIZE: f32 = 26.0;
pub const LABEL_SIZE: f32 = 17.0;
pub const SMALL_SIZE: f32 = 14.0;
pub const ROW_HEIGHT: f32 = 26.0;
pub const MARGIN: f32 = 14.0;

pub const TITLE_WIDTH: f32 = 520.0;
pub const TITLE_HEIGHT: f32 = 300.0;
pub const MENU_WIDTH: f32 = 340.0;
pub const MENU_HEIGHT: f32 = 250.0;
pub const SHOP_WIDTH: f32 = 460.0;
pub const SHOP_HEIGHT: f32 = 300.0;
pub const JOBS_WIDTH: f32 = 560.0;
pub const JOBS_HEIGHT: f32 = 380.0;
pub const PARTY_WIDTH: f32 = 760.0;
pub const PARTY_HEIGHT: f32 = 84.0;
pub const HERO_CARD_WIDTH: f32 = 180.0;
pub const BANNER_WIDTH: f32 = 620.0;
pub const BANNER_HEIGHT: f32 = 34.0;
pub const FOE_WIDTH: f32 = 300.0;
pub const FOE_HEIGHT: f32 = 150.0;
pub const COMMAND_WIDTH: f32 = 220.0;
pub const COMMAND_HEIGHT: f32 = 230.0;

pub const SETTLED: u64 = 120;
