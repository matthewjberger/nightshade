use nalgebra_glm::{Vec2, Vec3, Vec4};
use nightshade_render::prelude::Twilight;

pub const TITLE: &str = "Farm";
pub const TAGLINE: &str = "A farm on the edge of the Hollow";

pub const ATLAS: &str = "diorama";
pub const SHEET: &str = "tiny_farm";
pub const SHADOW: &str = "shadow";
pub const ATLAS_SIZE: u32 = 512;
pub const DUNGEON_SHEET: &str = "tiny_dungeon";
pub const DUNGEON_SHEET_BYTES: &[u8] = include_bytes!("../assets/tiny_dungeon/tilemap_packed.png");

pub const SHEET_BYTES: &[u8] = include_bytes!("../assets/tiny_farm/tilemap_packed.png");
pub const SHEET_COLUMNS: u32 = 12;
pub const SHEET_ROWS: u32 = 11;
pub const SHEET_TILES: usize = (SHEET_COLUMNS * SHEET_ROWS) as usize;
pub const TILE_TEXELS: u32 = 16;

pub const ICON_PAGE: u32 = 512;
pub const ICON_SCALE: u32 = 2;
pub const ICON_CELL: u32 = TILE_TEXELS * ICON_SCALE;
pub const CUSTOM_ICON_START: u32 = SHEET_TILES as u32;
pub const CUSTOM_ICON_ROW: u32 = SHEET_ROWS;

pub const BLOCK_TEXELS: u32 = 16;
pub const GRASS_TEXTURE: &str = "block_grass";
pub const DIRT_TEXTURE: &str = "block_dirt";
pub const SAND_TEXTURE: &str = "block_sand";
pub const STONE_TEXTURE: &str = "block_stone";
pub const TILLED_TEXTURE: &str = "block_tilled";
pub const WET_TILLED_TEXTURE: &str = "block_wet_tilled";
pub const PLANK_TEXTURE: &str = "block_plank";
pub const RED_PLANK_TEXTURE: &str = "block_red_plank";
pub const PLASTER_TEXTURE: &str = "block_plaster";
pub const STONE_WALL_TEXTURE: &str = "block_stone_wall";
pub const ROOF_RED_TEXTURE: &str = "block_roof_red";
pub const ROOF_DARK_TEXTURE: &str = "block_roof_dark";
pub const ROOF_GREEN_TEXTURE: &str = "block_roof_green";
pub const DOOR_TEXTURE: &str = "block_door";
pub const WINDOW_TEXTURE: &str = "block_window";

pub const GRASS_MATERIAL: &str = "grass";
pub const DIRT_MATERIAL: &str = "dirt";
pub const SAND_MATERIAL: &str = "sand";
pub const STONE_MATERIAL: &str = "stone";
pub const TILLED_MATERIAL: &str = "tilled";
pub const WET_TILLED_MATERIAL: &str = "wet_tilled";
pub const WATER_MATERIAL: &str = "water";
pub const SEA_MATERIAL: &str = "sea";
pub const POST_MATERIAL: &str = "post";
pub const LAMP_MATERIAL: &str = "lamp";
pub const PLANK_MATERIAL: &str = "plank";
pub const RED_PLANK_MATERIAL: &str = "red_plank";
pub const PLASTER_MATERIAL: &str = "plaster";
pub const TIMBER_MATERIAL: &str = "timber";
pub const STONE_WALL_MATERIAL: &str = "stone_wall";
pub const ROOF_RED_MATERIAL: &str = "roof_red";
pub const ROOF_DARK_MATERIAL: &str = "roof_dark";
pub const ROOF_GREEN_MATERIAL: &str = "roof_green";
pub const DOOR_MATERIAL: &str = "door";
pub const WINDOW_MATERIAL: &str = "window";
pub const FORGE_MATERIAL: &str = "forge";
pub const IRON_MATERIAL: &str = "iron";

pub const GRASS_COLOR: [u8; 4] = [108, 176, 84, 255];
pub const GRASS_DARK: [u8; 4] = [92, 156, 72, 255];
pub const GRASS_LIGHT: [u8; 4] = [132, 198, 105, 255];
pub const DIRT_COLOR: [u8; 4] = [128, 92, 62, 255];
pub const DIRT_DARK: [u8; 4] = [104, 72, 48, 255];
pub const DIRT_LIGHT: [u8; 4] = [150, 112, 78, 255];
pub const SAND_COLOR: [u8; 4] = [214, 196, 140, 255];
pub const SAND_DARK: [u8; 4] = [196, 176, 120, 255];
pub const SAND_LIGHT: [u8; 4] = [230, 214, 160, 255];
pub const STONE_COLOR: [u8; 4] = [146, 142, 136, 255];
pub const STONE_ALT: [u8; 4] = [160, 154, 146, 255];
pub const STONE_MORTAR: [u8; 4] = [96, 92, 90, 255];
pub const TILLED_COLOR: [u8; 4] = [96, 66, 42, 255];
pub const TILLED_FURROW: [u8; 4] = [124, 88, 58, 255];
pub const WET_TILLED_COLOR: [u8; 4] = [62, 42, 28, 255];
pub const WET_TILLED_FURROW: [u8; 4] = [86, 60, 40, 255];
pub const PLANK_COLOR: [u8; 4] = [168, 122, 72, 255];
pub const PLANK_DARK: [u8; 4] = [124, 86, 48, 255];
pub const PLANK_SEAM: [u8; 4] = [86, 58, 34, 255];
pub const RED_PLANK_COLOR: [u8; 4] = [176, 62, 50, 255];
pub const RED_PLANK_DARK: [u8; 4] = [140, 46, 38, 255];
pub const RED_PLANK_SEAM: [u8; 4] = [92, 30, 26, 255];
pub const PLASTER_COLOR: [u8; 4] = [232, 220, 192, 255];
pub const PLASTER_DARK: [u8; 4] = [214, 200, 170, 255];
pub const PLASTER_LIGHT: [u8; 4] = [244, 236, 214, 255];
pub const TIMBER_COLOR: [f32; 4] = [0.36, 0.24, 0.14, 1.0];
pub const WALL_STONE_COLOR: [u8; 4] = [128, 124, 122, 255];
pub const WALL_STONE_ALT: [u8; 4] = [148, 142, 138, 255];
pub const WALL_STONE_MORTAR: [u8; 4] = [78, 74, 74, 255];
pub const ROOF_RED_COLOR: [u8; 4] = [170, 70, 52, 255];
pub const ROOF_RED_DARK: [u8; 4] = [128, 50, 40, 255];
pub const ROOF_DARK_COLOR: [u8; 4] = [78, 70, 82, 255];
pub const ROOF_DARK_DARK: [u8; 4] = [54, 48, 60, 255];
pub const ROOF_GREEN_COLOR: [u8; 4] = [84, 132, 78, 255];
pub const ROOF_GREEN_DARK: [u8; 4] = [58, 98, 56, 255];
pub const DOOR_COLOR: [u8; 4] = [116, 74, 40, 255];
pub const DOOR_DARK: [u8; 4] = [80, 50, 28, 255];
pub const DOOR_KNOB: [u8; 4] = [220, 190, 90, 255];
pub const WINDOW_GLASS: [u8; 4] = [150, 196, 230, 255];
pub const WINDOW_GLASS_LIGHT: [u8; 4] = [196, 226, 244, 255];
pub const WINDOW_FRAME: [u8; 4] = [86, 58, 34, 255];
pub const WINDOW_GLOW: Vec3 = Vec3::new(1.6, 1.05, 0.45);
pub const FORGE_GLOW: [f32; 3] = [6.0, 2.2, 0.5];

pub const TERRAIN_COLUMNS: u32 = 64;
pub const TERRAIN_ROWS: u32 = 48;
pub const TERRAIN_ORIGIN: Vec2 = Vec2::new(-32.0, -24.0);
pub const MIN_HEIGHT: u32 = 1;
pub const MAX_HEIGHT: u32 = 5;
pub const BRIDGE_HEIGHT: u32 = 2;

pub const CAP_THICKNESS: f32 = 0.25;

pub const WATER_LEVEL: f32 = 1.55;
pub const SEA_LEVEL: f32 = 0.9;
pub const SEA_SIZE: f32 = 600.0;
pub const FOG_START: f32 = 22.0;
pub const FOG_END: f32 = 60.0;
pub const RAIN_FOG_START: f32 = 10.0;
pub const RAIN_FOG_END: f32 = 34.0;

pub const VILLAGE_MIN: Vec2 = Vec2::new(-22.0, -15.0);
pub const VILLAGE_MAX: Vec2 = Vec2::new(18.0, -9.0);
pub const VILLAGE_HEIGHT: u32 = 3;
pub const PEN_MIN: Vec2 = Vec2::new(3.0, -9.0);
pub const PEN_MAX: Vec2 = Vec2::new(11.0, -3.0);
pub const PEN_HEIGHT: u32 = 3;
pub const PLOT_ORIGIN: Vec2 = Vec2::new(-10.0, -8.0);
pub const PLOT_COLUMNS: u32 = 4;
pub const PLOT_ROWS: u32 = 3;
pub const PLOT_HEIGHT: u32 = 3;
pub const HUT_MIN: Vec2 = Vec2::new(11.0, 8.0);
pub const HUT_MAX: Vec2 = Vec2::new(17.0, 13.0);
pub const HUT_HEIGHT: u32 = 2;
pub const CLIFF_MIN: Vec2 = Vec2::new(-32.0, -24.0);
pub const CLIFF_MAX: Vec2 = Vec2::new(32.0, -16.0);
pub const CLIFF_HEIGHT: u32 = 5;
pub const FOREST_MIN: Vec2 = Vec2::new(-32.0, -16.0);
pub const FOREST_MAX: Vec2 = Vec2::new(-18.0, 24.0);
pub const QUARRY_MIN: Vec2 = Vec2::new(20.0, -16.0);
pub const QUARRY_MAX: Vec2 = Vec2::new(32.0, -2.0);
pub const QUARRY_HEIGHT: u32 = 5;
pub const RAMP_MIN: Vec2 = Vec2::new(18.0, -12.0);
pub const RAMP_MAX: Vec2 = Vec2::new(20.0, -8.0);
pub const RAMP_HEIGHT: u32 = 4;
pub const LAKE_MIN: Vec2 = Vec2::new(6.0, 10.0);
pub const LAKE_MAX: Vec2 = Vec2::new(32.0, 24.0);
pub const DOCK_MIN: Vec2 = Vec2::new(13.0, 13.0);
pub const DOCK_MAX: Vec2 = Vec2::new(15.0, 17.0);
pub const MEADOW_MIN: Vec2 = Vec2::new(-18.0, 10.0);
pub const MEADOW_MAX: Vec2 = Vec2::new(6.0, 24.0);
pub const MEADOW_MIN_HEIGHT: u32 = 2;
pub const MEADOW_MAX_HEIGHT: u32 = 3;
pub const FOREST_TREES: u32 = 130;
pub const MEADOW_BUSHES: u32 = 28;
pub const QUARRY_ROCKS: u32 = 26;
pub const REGROW_DAYS: u32 = 3;
pub const STUMP_WOOD: u32 = 2;
pub const PICK_RADIUS: f32 = 1.2;
pub const BERRIES_PER_BUSH: u32 = 2;

pub const WELL: Vec2 = Vec2::new(0.5, -6.5);
pub const CAMPFIRE: Vec2 = Vec2::new(2.5, 11.5);
pub const ANVIL: Vec2 = Vec2::new(-0.6, -10.4);
pub const WOOD_PILE: Vec2 = Vec2::new(17.3, -10.4);
pub const FORGE_LIGHT_OFFSET: Vec3 = Vec3::new(0.0, 1.0, 0.9);

pub const FARMER_START: Vec2 = Vec2::new(-3.5, -5.5);

pub const BRAM: &str = "Bram";
pub const MIRA: &str = "Mira";
pub const WREN: &str = "Wren";
pub const LINA: &str = "Lina";
pub const TAM: &str = "Tam";
pub const PIP: &str = "Pip";
pub const OSK: &str = "Osk";
pub const DOV: &str = "Dov";
pub const NELL: &str = "Nell";
pub const JUNO: &str = "Juno";

pub const LAYER_SHADOW: i32 = 0;
pub const LAYER_STANDING: i32 = 1;
pub const SHADOW_ALPHA: f32 = 0.55;
pub const SHADOW_LIFT: f32 = 0.02;
pub const SHADOW_FORWARD: f32 = 0.12;

pub const PINE_TOP: u32 = 3;
pub const PINE_BOTTOM: u32 = 15;
pub const DEAD_TREE_TOP: u32 = 2;
pub const DEAD_TREE_BOTTOM: u32 = 14;
pub const SMALL_PINE: u32 = 27;
pub const STUMP: u32 = 38;
pub const BUSH: u32 = 39;
pub const BERRY_BUSH: u32 = 78;
pub const TUFTS: [u32; 4] = [64, 65, 80, 52];
pub const SUNFLOWER: u32 = 83;
pub const ROCKS: u32 = 77;
pub const ROCK_PILE: u32 = 89;
pub const HAY: u32 = 96;
pub const HAY_BALE: u32 = 97;
pub const BARREL: u32 = 85;
pub const SACK: u32 = 74;
pub const BASKET: u32 = 75;
pub const CRATE: u32 = 76;
pub const FARMER: u32 = 108;
pub const BRAM_TILE: u32 = 87;
pub const MIRA_TILE: u32 = 99;
pub const WREN_TILE: u32 = 86;
pub const LINA_TILE: u32 = 111;
pub const TAM_TILE: u32 = 88;
pub const PIP_TILE: u32 = 98;
pub const OSK_TILE: u32 = 100;
pub const DOV_TILE: u32 = 85;
pub const NELL_TILE: u32 = 112;
pub const JUNO_TILE: u32 = 84;
pub const SHEEP: u32 = 120;
pub const COW: u32 = 121;
pub const CHICKEN: u32 = 122;
pub const MILK_PAIL: u32 = 123;
pub const EGG: u32 = 125;
pub const CARROT_STAGES: [u32; 4] = [5, 6, 7, 8];
pub const EGGPLANT_STAGES: [u32; 4] = [17, 18, 19, 20];
pub const TOMATO_STAGES: [u32; 4] = [41, 42, 43, 44];
pub const WHEAT_STAGES: [u32; 4] = [65, 66, 67, 68];
pub const CARROT_SEED_ICON: u32 = 9;
pub const EGGPLANT_SEED_ICON: u32 = 21;
pub const TOMATO_SEED_ICON: u32 = 45;
pub const WHEAT_SEED_ICON: u32 = 69;
pub const HOE_ICON: u32 = 88;
pub const AXE_ICON: u32 = 87;
pub const SHOVEL_ICON: u32 = 86;
pub const BUCKET_ICON: u32 = 72;
pub const WATERING_CAN_ICON: u32 = 73;
pub const ROD_ICON: u32 = CUSTOM_ICON_START;
pub const WOOD_ICON: u32 = CUSTOM_ICON_START + 1;
pub const FISH_ICON: u32 = CUSTOM_ICON_START + 2;
pub const BREAD_ICON: u32 = CUSTOM_ICON_START + 3;
pub const ORE_ICON: u32 = CUSTOM_ICON_START + 4;

pub const FARMER_SPEED: f32 = 3.5;
pub const STRIDE_RATE: f32 = 14.0;
pub const STRIDE_BOUNCE: f32 = 0.12;
pub const STEP_SMOOTHING: f32 = 12.0;

pub const CAMERA_HEIGHT: f32 = 8.0;
pub const CAMERA_REACH: f32 = 14.0;
pub const CAMERA_FOV_DEGREES: f32 = 32.0;
pub const CAMERA_NEAR: f32 = 0.1;
pub const CAMERA_FAR: f32 = 80.0;
pub const CAMERA_FOLLOW: f32 = 5.0;
pub const CAMERA_TURN: f32 = 9.0;

pub const START_HOUR: f32 = 8.0;
pub const DAY_SPEED: f32 = 0.1;
pub const TWILIGHT: Twilight = Twilight {
    dawn_start: 5.5,
    dawn_end: 7.0,
    dusk_start: 17.5,
    dusk_end: 19.5,
};
pub const WAKE_HOUR: f32 = 6.5;
pub const BED_HOUR: f32 = 21.0;
pub const SLEEP_FROM: f32 = 19.5;
pub const SLEEP_UNTIL: f32 = 5.0;
pub const MORNING_HOUR: f32 = 6.0;
pub const NIGHT_EXPOSURE_EV: f32 = -0.9;
pub const BLOOM_DAY: f32 = 0.35;
pub const BLOOM_NIGHT: f32 = 0.42;

pub const SKY_DAY_HORIZON: Vec3 = Vec3::new(0.74, 0.8, 0.9);
pub const SKY_DUSK_HORIZON: Vec3 = Vec3::new(0.98, 0.58, 0.34);
pub const SKY_NIGHT_HORIZON: Vec3 = Vec3::new(0.05, 0.06, 0.13);
pub const AMBIENT_DAY: Vec3 = Vec3::new(1.0, 1.0, 1.0);
pub const AMBIENT_DUSK: Vec3 = Vec3::new(0.2, 0.06, -0.06);
pub const AMBIENT_NIGHT: Vec3 = Vec3::new(0.12, 0.14, 0.22);
pub const AMBIENT_FLOOR_ELEVATION: f32 = -0.55;
pub const SPRITE_LIGHT_GAIN: f32 = 0.5;
pub const SPRITE_LIGHT_CAP: f32 = 1.35;
pub const SEA_DAY: Vec3 = Vec3::new(0.14, 0.3, 0.48);
pub const SUN_SHADOW_SOFTNESS: f32 = 0.5;
pub const MOON_POSITION: Vec3 = Vec3::new(14.0, 18.0, -6.0);
pub const MOON_COLOR: Vec3 = Vec3::new(0.55, 0.66, 0.95);
pub const MOON_STRENGTH: f32 = 0.45;

pub const LANTERN_POST_HEIGHT: f32 = 1.2;
pub const LANTERN_LAMP_SIZE: f32 = 0.3;
pub const LANTERN_COLOR: Vec3 = Vec3::new(1.0, 0.72, 0.42);
pub const LANTERN_STRENGTH: f32 = 11.0;
pub const LANTERN_RANGE: f32 = 9.0;
pub const CAMPFIRE_COLOR: Vec3 = Vec3::new(1.0, 0.55, 0.25);
pub const CAMPFIRE_STRENGTH: f32 = 10.0;
pub const FORGE_LIGHT_COLOR: Vec3 = Vec3::new(1.0, 0.45, 0.15);
pub const FORGE_STRENGTH: f32 = 8.0;

pub const CLEAR_SHARE: f32 = 0.36;
pub const CLOUDY_SHARE: f32 = 0.18;
pub const FOG_SHARE: f32 = 0.12;
pub const WIND_SHARE: f32 = 0.14;
pub const RAIN_SHARE: f32 = 0.13;
pub const FOG_HOURS: (f32, f32) = (5.5, 10.5);
pub const FOG_THICK_START: f32 = 4.0;
pub const FOG_THICK_END: f32 = 16.0;
pub const CLOUD_EXPOSURE_EV: f32 = -0.22;
pub const STORM_EXPOSURE_EV: f32 = -0.25;
pub const LEAF_RATE: f32 = 70.0;
pub const LEAF_SPEED: f32 = 4.0;
pub const SWAY: f32 = 0.05;
pub const RAIN_EARLIEST: f32 = 7.0;
pub const RAIN_LATEST: f32 = 16.0;
pub const RAIN_MIN_HOURS: f32 = 2.0;
pub const RAIN_MAX_HOURS: f32 = 6.0;
pub const RAIN_RATE: f32 = 900.0;
pub const RAIN_HEIGHT: f32 = 9.0;
pub const RAIN_REACH: Vec3 = Vec3::new(13.0, 0.3, 10.0);
pub const RAIN_FADE: f32 = 0.4;
pub const RAIN_EXPOSURE_EV: f32 = -0.35;
pub const LIGHTNING_GAP_MIN: f32 = 6.0;
pub const LIGHTNING_GAP_MAX: f32 = 18.0;
pub const LIGHTNING_EV: f32 = 1.6;
pub const LIGHTNING_SECONDS: f32 = 0.12;

pub const START_COINS: u32 = 20;
pub const HARVEST_YIELD: u32 = 2;
pub const EGG_INTERVAL: f32 = 40.0;
pub const EGG_PICKUP_RADIUS: f32 = 0.7;
pub const TALK_RADIUS: f32 = 1.8;
pub const DOOR_RADIUS: f32 = 0.9;
pub const CHOP_RADIUS: f32 = 1.4;
pub const DIG_RADIUS: f32 = 1.4;
pub const MILK_RADIUS: f32 = 1.6;
pub const TREE_HITS: u32 = 3;
pub const SMALL_TREE_HITS: u32 = 2;
pub const WOOD_PER_TREE: u32 = 3;
pub const WOOD_PER_SMALL_TREE: u32 = 1;
pub const STONE_PER_ROCKS: u32 = 2;
pub const SWING_SECONDS: f32 = 0.34;
pub const SWING_TOOL_SIZE: f32 = 0.62;
pub const SWING_OFFSET: Vec3 = Vec3::new(0.34, 0.42, 0.08);
pub const SWING_RAISED: f32 = 1.3;
pub const SWING_ARC: f32 = 2.2;
pub const FISH_WAIT_MIN: f32 = 2.0;
pub const FISH_WAIT_MAX: f32 = 6.0;
pub const BITE_WINDOW: f32 = 0.9;
pub const ACT_COOLDOWN: f32 = 0.32;
pub const LOG_TILE: u32 = 26;
pub const PICKUP_SIZE: f32 = 0.44;
pub const PICKUP_SPREAD: f32 = 0.6;
pub const PICKUP_REACH: f32 = 0.85;
pub const SHAKE_SECONDS: f32 = 0.35;
pub const SHAKE_RATE: f32 = 60.0;
pub const SHAKE_LEAN: f32 = 0.12;
pub const TYPEWRITER_RATE: f32 = 45.0;
pub const TOAST_SECONDS: f32 = 2.5;

pub const SPARKLE_COUNT: u32 = 40;
pub const LEAF_COUNT: u32 = 24;
pub const FIREFLY_RATE: f32 = 6.0;
pub const FIREFLY_SIZE: f32 = 0.2;
pub const FIREFLY_SPOTS: [(f32, f32); 12] = [
    (-14.0, 6.0),
    (-6.0, 8.0),
    (2.0, 9.0),
    (10.0, 7.0),
    (-9.0, -5.0),
    (7.0, -3.0),
    (-3.0, -1.0),
    (4.0, 1.0),
    (12.0, 12.0),
    (20.0, 14.0),
    (-8.0, 14.0),
    (-24.0, 2.0),
];
pub const FIREWORKS_HOUR: f32 = 20.5;
pub const FIREWORKS_END: f32 = 2.0;
pub const FIREWORKS_INTERVAL: f32 = 2.2;
pub const FIREWORKS_LAUNCH: Vec3 = Vec3::new(-1.0, 1.6, 2.5);
pub const FIREWORKS_SPEED_MIN: f32 = 9.0;
pub const FIREWORKS_SPEED_MAX: f32 = 11.0;
pub const FIREWORKS_FUSE: f32 = 0.9;
pub const FIREWORK_PARTICLES: u32 = 220;
pub const FIREWORK_SCALE: f32 = 0.14;
pub const FIREWORK_SPARK_SIZE: f32 = 0.09;
pub const FIREWORK_COLORS: [Vec3; 5] = [
    Vec3::new(1.0, 0.35, 0.3),
    Vec3::new(1.0, 0.85, 0.3),
    Vec3::new(0.4, 0.9, 1.0),
    Vec3::new(0.7, 0.4, 1.0),
    Vec3::new(0.4, 1.0, 0.5),
];

pub const PANEL_COLOR: Vec4 = Vec4::new(0.08, 0.07, 0.1, 0.9);
pub const TITLE_PANEL_COLOR: Vec4 = Vec4::new(0.13, 0.09, 0.07, 0.95);
pub const TITLE_ICON: f32 = 84.0;
pub const TITLE_STRIP: [u32; 5] = [108, 44, 32, 122, 120];
pub const PANEL_BORDER: Vec4 = Vec4::new(0.85, 0.7, 0.45, 0.9);
pub const SLOT_COLOR: Vec4 = Vec4::new(0.12, 0.11, 0.14, 0.92);
pub const SLOT_SELECTED_COLOR: Vec4 = Vec4::new(0.36, 0.28, 0.16, 0.96);
pub const ICON_COLOR: Vec4 = Vec4::new(1.0, 1.0, 1.0, 1.0);
pub const ICON_DIM_COLOR: Vec4 = Vec4::new(0.5, 0.5, 0.5, 0.35);
pub const TEXT_COLOR: Vec4 = Vec4::new(0.96, 0.93, 0.86, 1.0);
pub const DIM_TEXT_COLOR: Vec4 = Vec4::new(0.7, 0.66, 0.6, 1.0);
pub const ACCENT_COLOR: Vec4 = Vec4::new(1.0, 0.82, 0.45, 1.0);
pub const DONE_COLOR: Vec4 = Vec4::new(0.6, 0.9, 0.6, 1.0);
pub const TIRED_COLOR: Vec4 = Vec4::new(0.95, 0.55, 0.45, 1.0);
pub const TITLE_SIZE: f32 = 38.0;
pub const HEADING_SIZE: f32 = 26.0;
pub const BODY_SIZE: f32 = 19.0;
pub const SMALL_SIZE: f32 = 16.0;
pub const TINY_SIZE: f32 = 13.0;
pub const DIALOGUE_WIDTH: f32 = 760.0;
pub const DIALOGUE_HEIGHT: f32 = 130.0;
pub const MENU_WIDTH: f32 = 560.0;
pub const SHOP_ROW_HEIGHT: f32 = 46.0;
pub const INVENTORY_HEIGHT: f32 = 730.0;
pub const CLEAR_COLOR: Vec4 = Vec4::new(0.0, 0.0, 0.0, 0.0);
pub const SCRIM_COLOR: Vec4 = Vec4::new(0.02, 0.02, 0.04, 0.66);
pub const MENU_ROW_COLOR: Vec4 = Vec4::new(0.14, 0.13, 0.17, 0.92);
pub const MENU_ROW_DIM: Vec4 = Vec4::new(0.1, 0.09, 0.12, 0.7);
pub const POCKET_WIDTH: f32 = 780.0;
pub const POCKET_CELL: f32 = 138.0;
pub const POCKET_CELL_HEIGHT: f32 = 78.0;
pub const POCKET_ICON: f32 = 40.0;
pub const POCKET_COLUMNS: usize = 5;
pub const SHOP_WIDTH: f32 = 880.0;
pub const SHOP_COLUMN: f32 = 406.0;
pub const MENU_BUTTON: f32 = 92.0;
pub const MENU_BUTTON_WIDE: f32 = 190.0;
pub const MENU_BUTTON_HEIGHT: f32 = 34.0;
pub const HOTBAR_SLOT: f32 = 60.0;
pub const HOTBAR_HEIGHT: f32 = 78.0;
pub const HOTBAR_ICON: f32 = 36.0;
pub const HOTBAR_SPACING: f32 = 6.0;

pub const SETTLED: u64 = 600;

pub const HOLLOW_Y: f32 = -80.0;
pub const HOLLOW_COLUMNS: u32 = 26;
pub const HOLLOW_ROWS: u32 = 20;
pub const HOLLOW_ORIGIN: Vec2 = Vec2::new(-13.0, -10.0);
pub const HOLLOW_FLOOR_SHARE: f32 = 0.42;
pub const HOLLOW_WALL_LEVELS: u32 = 2;
pub const HOLLOW_WALL_HEIGHT: f32 = HOLLOW_WALL_LEVELS as f32;
pub const HOLLOW_ROCK_SIZE: f32 = 240.0;
pub const HOLLOW_ROOF_HEIGHT: f32 = 12.0;
pub const HOLLOW_MIN: Vec2 = Vec2::new(-1.0, -19.0);
pub const HOLLOW_MAX: Vec2 = Vec2::new(4.0, -15.0);
pub const HOLLOW_DOOR: Vec2 = Vec2::new(1.5, -18.5);
pub const HOLLOW_DOOR_SPOT: Vec2 = Vec2::new(1.5, -17.0);
pub const HOLLOW_LANTERNS: [Vec2; 2] = [Vec2::new(-0.5, -16.5), Vec2::new(3.5, -16.5)];
pub const HOLLOW_FRAME_REACH: f32 = 1.5;
pub const HOLLOW_FRAME_LEVELS: u32 = 2;
pub const HOLLOW_FRAME_HEIGHT: f32 = HOLLOW_FRAME_LEVELS as f32;
pub const HOLLOW_FRAME_WIDTH: u32 = 4;
pub const HOLLOW_AMBIENT: Vec3 = Vec3::new(0.26, 0.28, 0.36);
pub const HOLLOW_FOG: Vec3 = Vec3::new(0.02, 0.02, 0.03);
pub const HOLLOW_FOG_START: f32 = 12.0;
pub const HOLLOW_FOG_END: f32 = 30.0;
pub const TORCH_EVERY: u32 = 6;
pub const TORCH_COLOR: Vec3 = Vec3::new(1.0, 0.6, 0.3);
pub const TORCH_STRENGTH: f32 = 7.0;
pub const TORCH_RANGE: f32 = 6.5;
pub const DUNGEON_WALL_SURFACES: [(&str, u32); 3] = [
    ("hollow_wall_block", 57),
    ("hollow_wall_split", 58),
    ("hollow_wall_stack", 59),
];
pub const DUNGEON_FLOOR_SURFACES: [(&str, u32); 3] = [
    ("hollow_floor_sand", 48),
    ("hollow_floor_grit", 49),
    ("hollow_floor_worn", 51),
];
pub const DUNGEON_CAP_SURFACE: (&str, u32) = ("hollow_wall_cap", 59);
pub const DUNGEON_CAP_HEIGHT: f32 = 0.22;
pub const DUNGEON_ROCK_MATERIAL: &str = "hollow_rock";
pub const DUNGEON_PROP_TILES: [u32; 3] = [73, 65, 64];
pub const DUNGEON_PROP_EVERY: u32 = 7;
pub const DOORWAY_LEFT: u32 = 10;
pub const TORCH_TILE: u32 = 125;
pub const HOLLOW_WALL_SHADE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
pub const HOLLOW_FLOOR_SHADE: [f32; 4] = [0.82, 0.78, 0.72, 1.0];
pub const LADDER_TILE: u32 = 63;
pub const WOOD_BED_TILE: u32 = 98;
pub const GREY_BED_TILE: u32 = 100;
pub const BLUE_BED_TILE: u32 = 110;
pub const YELLOW_BED_TILE: u32 = 112;
pub const PAIL_TILE: u32 = 72;
pub const WATER_PAIL_TILE: u32 = 73;
pub const HAY_SACK_TILE: u32 = 74;
pub const BASKET_TILE: u32 = 75;
pub const CRATE_TILE: u32 = 76;
pub const ANVIL_TILE: u32 = 77;
pub const BARREL_TILE: u32 = 85;
pub const SACK_TILE: u32 = 125;

pub const INDOOR_Y: f32 = -180.0;
pub const INDOOR_WALL_LEVELS: u32 = 2;
pub const INDOOR_DOOR_WIDTH: f32 = 1.4;
pub const INDOOR_DOOR_HEIGHT: f32 = 1.8;
pub const INDOOR_DOOR_THICKNESS: f32 = 0.14;
pub const INDOOR_ROOF_HEIGHT: f32 = 3.0;
pub const INDOOR_AMBIENT: Vec3 = Vec3::new(0.5, 0.46, 0.42);
pub const INDOOR_FOG: Vec3 = Vec3::new(0.06, 0.05, 0.05);
pub const INDOOR_FOG_START: f32 = 14.0;
pub const INDOOR_FOG_END: f32 = 40.0;
pub const INDOOR_LAMP_COLOR: Vec3 = Vec3::new(1.0, 0.85, 0.62);
pub const INDOOR_LAMP_STRENGTH: f32 = 9.0;
pub const INDOOR_LAMP_RANGE: f32 = 9.0;
pub const EXIT_RADIUS: f32 = 0.9;
pub const DOOR_STEP: f32 = 0.9;
pub const BED_RADIUS: f32 = 1.0;
pub const COUNTER_RADIUS: f32 = 1.1;
pub const CHEST_CLOSED: u32 = 89;
pub const CHEST_OPEN: u32 = 91;
pub const MIMIC_TILE: u32 = 92;
pub const COIN_TILE: u32 = 101;
pub const SLIME_TILE: u32 = 108;
pub const CRAB_TILE: u32 = 110;
pub const BAT_TILE: u32 = 120;
pub const GHOST_TILE: u32 = 121;
pub const SPIDER_TILE: u32 = 122;
pub const RAT_TILE: u32 = 123;
pub const WOLF_TILE: u32 = 124;
pub const DAGGER_TILE: u32 = 103;
pub const SWORD_TILE: u32 = 104;
pub const AXE_TILE: u32 = 118;
pub const HAMMER_TILE: u32 = 117;
pub const POTION_HEAL: u32 = 115;
pub const POTION_STRENGTH: u32 = 114;
pub const POTION_SPEED: u32 = 116;
pub const POTION_GRIT: u32 = 113;
pub const HERO_MAX_HP: u32 = 10;
pub const ATTACK_RANGE: f32 = 1.35;
pub const ATTACK_COOLDOWN: f32 = 0.45;
pub const HURT_SECONDS: f32 = 0.16;
pub const KNOCKBACK: f32 = 0.55;
pub const INVULNERABLE_SECONDS: f32 = 0.7;
pub const AGGRO_RANGE: f32 = 5.5;
pub const BITE_RANGE: f32 = 0.75;
pub const BITE_COOLDOWN: f32 = 1.0;
pub const XP_PER_LEVEL: u32 = 12;
pub const LEVEL_HP: u32 = 2;
pub const BUFF_SECONDS: f32 = 60.0;
pub const HEAL_AMOUNT: u32 = 5;
pub const STRENGTH_BONUS: u32 = 2;
pub const SPEED_BONUS: f32 = 0.4;
pub const GRIT_BONUS: u32 = 1;
pub const ENEMIES_BASE: u32 = 4;
pub const ENEMIES_PER_DEPTH: u32 = 2;
pub const CHESTS_PER_LEVEL: u32 = 2;
pub const POTIONS_PER_LEVEL: u32 = 2;
pub const MIMIC_CHANCE: f32 = 0.2;
pub const BOSS_EVERY: u32 = 5;
pub const CHEST_COINS: u32 = 12;
pub const CHEST_COINS_PER_DEPTH: u32 = 6;
pub const DROP_POTION_CHANCE: f32 = 0.22;
pub const DEATH_COIN_SHARE: f32 = 0.5;
pub const SURFACE_REGEN: f32 = 0.35;
pub const PICKUP_RADIUS: f32 = 0.6;

pub const MAX_STAMINA: f32 = 100.0;
pub const TIRED_STAMINA: f32 = 20.0;
pub const MAX_TOOL_LEVEL: u32 = 3;
pub const TOOL_LEVEL_DISCOUNT: f32 = 0.15;
pub const TEMPER_ORE: u32 = 3;
pub const TEMPER_COINS: u32 = 40;
pub const CHEST_ORE: u32 = 2;
pub const DROP_ORE_CHANCE: f32 = 0.4;
pub const SAVE_FOLDER: &str = "nightshade-farm";
pub const SAVE_NAME: &str = "farm";
pub const SAVE_VERSION: u32 = 1;
pub const SAVE_SLOT: u32 = 0;
pub const MEAL_BUTTON_WIDTH: f32 = 138.0;
pub const MEAL_COLUMNS: usize = 5;

pub const VILLAGE_PROPS: [(Vec2, u32); 18] = [
    (Vec2::new(-1.5, -7.5), CRATE),
    (Vec2::new(-0.5, -7.5), BARREL),
    (Vec2::new(2.5, -7.5), SACK),
    (Vec2::new(3.5, -7.5), CRATE),
    (Vec2::new(-2.5, -8.5), BASKET),
    (Vec2::new(4.5, -8.5), BARREL),
    (Vec2::new(-13.5, -10.5), SUNFLOWER),
    (Vec2::new(-12.5, -10.5), SUNFLOWER),
    (Vec2::new(-5.5, -10.5), BUSH),
    (Vec2::new(0.5, -10.5), BUSH),
    (Vec2::new(6.5, -10.5), HAY_BALE),
    (Vec2::new(12.5, -10.5), HAY),
    (Vec2::new(16.5, -10.5), CRATE),
    (Vec2::new(17.5, -10.5), BARREL),
    (Vec2::new(-17.5, -10.5), BARREL),
    (Vec2::new(-16.5, -10.5), CRATE),
    (Vec2::new(-19.5, -10.5), SACK),
    (Vec2::new(-8.5, -10.5), BASKET),
];

pub const MIDDAY_HOUR: f32 = 12.0;
pub const EVENING_HOUR: f32 = 17.0;
pub const MARKET: Vec2 = Vec2::new(1.0, -7.5);
pub const TAVERN_FRONT: Vec2 = Vec2::new(-19.5, -10.5);
pub const CHAPEL_FRONT: Vec2 = Vec2::new(-10.5, -10.5);
