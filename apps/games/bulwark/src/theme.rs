use nalgebra_glm::Vec3;

pub const TITLE: &str = "Bulwark";

pub const ATLAS: &str = "bulwark";
pub const RECORD_FOLDER: &str = "nightshade";
pub const RECORD_NAME: &str = "bulwark";
pub const RECORD_VERSION: u32 = 1;
pub const RECORD_SLOT: u32 = 0;
pub const ATLAS_SIZE: u32 = 2048;
pub const PIXELS_PER_UNIT: f32 = 66.0;

pub const MARKER: &str = "marker";
pub const RING: &str = "ring";
pub const BEAD: &str = "bead";
pub const BAR: &str = "bar";
pub const SHADE: &str = "shade";
pub const BACKDROP: &str = "backdrop";

pub const GROUND_ABOVE_BOTTOM: f32 = 66.0;
pub const SHARD_ABOVE_BOTTOM: f32 = 7.0;

pub const CELL_SIDES: usize = 4;
pub const HALF_STEP_ACROSS: f32 = 1.0;
pub const HALF_STEP_INTO: f32 = 0.5;

pub const LAYER_BACKDROP: i32 = -1;
pub const LAYER_GROUND: i32 = 0;
pub const LAYER_MARKER: i32 = 1;
pub const LAYER_STANDING: i32 = 2;
pub const LAYER_BEAD: i32 = 3;

pub const MARKER_TEXELS: u32 = 132;
pub const RING_TEXELS: u32 = 256;
pub const BEAD_TEXELS: u32 = 12;
pub const BAR_TEXELS: u32 = 8;
pub const SHADE_TEXELS: u32 = 64;
pub const BACKDROP_TEXELS: u32 = 256;

pub const MARKER_FREE: [f32; 4] = [0.58, 0.98, 0.62, 0.62];
pub const MARKER_BLOCKED: [f32; 4] = [0.98, 0.38, 0.36, 0.5];
pub const MARKER_CHOSEN: [f32; 4] = [0.99, 0.86, 0.34, 0.72];
pub const RING_COLOR: [f32; 4] = [0.72, 0.92, 1.0, 0.3];
pub const GATE_COLOR: [f32; 4] = [0.99, 0.62, 0.36, 1.0];
pub const GATE_LOW: f32 = 0.28;
pub const GATE_HIGH: f32 = 0.72;
pub const GATE_RATE: f32 = 2.4;
pub const GHOST_COLOR: [f32; 4] = [1.0, 1.0, 1.0, 0.45];
pub const BEAD_CORE: [u8; 4] = [255, 248, 218, 255];
pub const BEAD_EDGE: [u8; 4] = [255, 194, 92, 255];

pub const BAR_COLOR: [f32; 4] = [0.36, 0.93, 0.44, 1.0];
pub const BAR_HURT: [f32; 4] = [0.99, 0.74, 0.26, 1.0];
pub const BAR_LOW: [f32; 4] = [0.97, 0.33, 0.31, 1.0];
pub const BAR_BACK_COLOR: [f32; 4] = [0.04, 0.04, 0.06, 0.66];
pub const BAR_WIDTH: f32 = 0.46;
pub const BAR_HEIGHT: f32 = 0.085;
pub const BAR_LIFT: f32 = 0.1;
pub const SHADE_COLOR: [f32; 4] = [0.05, 0.1, 0.06, 0.5];
pub const BACKDROP_WIDTH: f32 = 74.0;
pub const BACKDROP_HEIGHT: f32 = 38.0;
pub const BACKDROP_CORE: [u8; 4] = [46, 62, 86, 255];
pub const BACKDROP_EDGE: [u8; 4] = [9, 11, 18, 255];
pub const VOID_COLOR: [f32; 4] = [0.035, 0.043, 0.07, 1.0];
pub const BOARD_SHADOW_WIDTH: f32 = 23.0;
pub const BOARD_SHADOW_HEIGHT: f32 = 15.0;
pub const BOARD_SHADOW_AT: f32 = -6.0;
pub const BOARD_SHADOW_COLOR: [f32; 4] = [0.0, 0.0, 0.0, 0.6];
pub const SHADE_WIDTH: f32 = 0.58;
pub const SHADE_HEIGHT: f32 = 0.27;

pub const CAMERA_AT: Vec3 = Vec3::new(0.0, -4.5, 12.0);
pub const CAMERA_HALF_HEIGHT: f32 = 6.7;
pub const CAMERA_NEAR: f32 = 0.1;
pub const CAMERA_FAR: f32 = 60.0;

pub const RISE_SECONDS: f32 = 0.28;
pub const RISE_FROM: f32 = 0.55;
pub const PACE_STEPS: [f32; 3] = [1.0, 2.0, 3.0];
pub const TOWER_SELL_SHARE: f32 = 0.6;
pub const BEAD_SPEED: f32 = 12.0;
pub const BEAD_LIFT: f32 = 0.62;

pub const BOB_HEIGHT: f32 = 0.07;
pub const BOB_RATE: f32 = 6.0;
pub const FLASH_SECONDS: f32 = 0.12;
pub const FLASH_COLOR: [f32; 4] = [2.4, 2.0, 2.4, 1.0];
pub const BREACH_FLASH: f32 = 0.28;
pub const BREACH_TINT: [f32; 4] = [2.0, 0.7, 0.6, 1.0];
pub const SHARD_COLOR: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

pub const STARTING_COIN: u32 = 95;
pub const STARTING_LIVES: u32 = 15;
pub const EARLY_BONUS: u32 = 4;
pub const SKIPPED_PURSE: u32 = 130;
pub const MEND_COST: u32 = 55;
pub const MEND_STEP: u32 = 25;
pub const FIRST_REST_SHARE: f32 = 2.0;
pub const WAVE_BOUNTY: u32 = 18;
pub const BOSS_HEALTH: f32 = 5.5;
pub const BOSS_SPEED: f32 = 0.68;
pub const BOSS_BOUNTY: u32 = 6;
pub const BOSS_SCALE: f32 = 1.45;
pub const SPLIT_COUNT: u32 = 2;
pub const SPLIT_HEALTH_SHARE: f32 = 0.3;
pub const SPLIT_SPEED_SHARE: f32 = 1.2;
pub const SPLIT_WARD_SHARE: f32 = 0.5;
pub const SPLIT_STEP: f32 = 0.09;
pub const CHILL_SECONDS: f32 = 1.5;
pub const CHILL_SHARE: f32 = 0.55;
pub const CHILL_COLOR: [f32; 4] = [0.6, 0.86, 1.25, 1.0];
pub const WARD_COLOR: [f32; 4] = [1.45, 1.05, 0.5, 1.0];
pub const WARD_LEAST_SHARE: f32 = 0.15;
pub const CHILL_WARD_SHARE: f32 = 0.4;
pub const PIERCE_SHARE: f32 = 0.25;
pub const SPLASH_REACH: f32 = 0.95;
pub const SPLASH_SHARE: f32 = 0.45;
pub const SHAKE_SECONDS: f32 = 0.36;
pub const SHAKE_REACH: f32 = 0.2;
pub const SHAKE_RATE: f32 = 42.0;
pub const WAVE_BOUNTY_STEP: u32 = 6;

pub const DIM_COLOR: [f32; 4] = [0.02, 0.03, 0.05, 0.58];
pub const HUD_PANEL: [f32; 4] = [0.05, 0.055, 0.085, 0.82];
pub const HUD_BORDER: [f32; 4] = [1.0, 1.0, 1.0, 0.13];
pub const HUD_LABEL: [f32; 4] = [0.6, 0.64, 0.73, 1.0];
pub const HUD_VALUE: [f32; 4] = [0.95, 0.96, 0.99, 1.0];
pub const HUD_COIN: [f32; 4] = [0.99, 0.84, 0.36, 1.0];
pub const HUD_LIVES: [f32; 4] = [0.98, 0.46, 0.44, 1.0];
pub const HUD_WAVE: [f32; 4] = [0.62, 0.86, 0.99, 1.0];
pub const HUD_GOOD: [f32; 4] = [0.54, 0.93, 0.6, 1.0];
pub const HUD_DIM: [f32; 4] = [0.44, 0.47, 0.55, 1.0];

pub const HUD_MARGIN: f32 = 16.0;
pub const CHIP_WIDTH: f32 = 118.0;
pub const CHIP_HEIGHT: f32 = 58.0;
pub const CHIP_GAP: f32 = 10.0;
pub const LABEL_SIZE: f32 = 13.0;
pub const VALUE_SIZE: f32 = 26.0;

pub const HOTBAR_WIDTH: f32 = 168.0;
pub const HOTBAR_HEIGHT: f32 = 96.0;
pub const HOTBAR_GAP: f32 = 12.0;
pub const HOTBAR_NAME_SIZE: f32 = 19.0;
pub const HOTBAR_BLURB_SIZE: f32 = 12.0;
pub const HOTBAR_IDLE: [f32; 4] = [0.05, 0.055, 0.085, 0.82];
pub const HOTBAR_CHOSEN: [f32; 4] = [0.13, 0.19, 0.25, 0.94];

pub const INSPECT_WIDTH: f32 = 244.0;
pub const INSPECT_HEIGHT: f32 = 258.0;

pub const BANNER_WIDTH: f32 = 496.0;
pub const BANNER_HEIGHT: f32 = 84.0;
pub const BANNER_TEXT_SIZE: f32 = 23.0;
pub const BANNER_NOTE_SIZE: f32 = 15.0;
pub const SEND_WIDTH: f32 = 148.0;
pub const PACE_WIDTH: f32 = 72.0;
pub const MEND_WIDTH: f32 = 128.0;
pub const HOLD_WIDTH: f32 = 84.0;
pub const SEND_HEIGHT: f32 = 40.0;

pub const MENU_WIDTH: f32 = 380.0;
pub const TITLE_HEIGHT: f32 = 410.0;
pub const PAUSE_HEIGHT: f32 = 264.0;
pub const OVER_HEIGHT: f32 = 256.0;
pub const OPTIONS_HEIGHT: f32 = 616.0;

pub const BURST_COUNT: u32 = 26;
pub const MUZZLE_COUNT: u32 = 7;
pub const DUST_COUNT: u32 = 18;
pub const BREACH_COUNT: u32 = 34;
pub const CLEARED_COUNT: u32 = 30;
pub const SPLASH_COUNT: u32 = 16;
pub const SPLASH_SPARK: [f32; 3] = [0.86, 0.82, 0.74];
pub const SHARD_SPARK: [f32; 3] = [0.76, 0.45, 0.92];
pub const COIN_SPARK: [f32; 3] = [1.0, 0.83, 0.32];
pub const BREACH_SPARK: [f32; 3] = [1.0, 0.36, 0.3];
