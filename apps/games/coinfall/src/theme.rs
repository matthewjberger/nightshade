use nightshade::prelude::{Vec2, Vec4};

pub const TITLE: &str = "Coinfall";
pub const SETTLED: u64 = 90;

pub const ATLAS: &str = "coinfall";
pub const ATLAS_SIZE: u32 = 512;
pub const TILE_SHEET: &str = "tiles";
pub const CHARACTER_SHEET: &str = "characters";
pub const BACKGROUND_SHEET: &str = "backgrounds";
pub const BLANK: &str = "blank";

pub const TILE_BYTES: &[u8] = include_bytes!("../assets/pixel_platformer/tilemap_packed.png");
pub const CHARACTER_BYTES: &[u8] =
    include_bytes!("../assets/pixel_platformer/tilemap-characters_packed.png");
pub const BACKGROUND_BYTES: &[u8] =
    include_bytes!("../assets/pixel_platformer/tilemap-backgrounds_packed.png");

pub const TILE_COLUMNS: u32 = 20;
pub const TILE_ROWS: u32 = 9;
pub const CHARACTER_COLUMNS: u32 = 9;
pub const CHARACTER_ROWS: u32 = 3;

pub const TILE: f32 = 1.0;
pub const TILE_SIZE: Vec2 = Vec2::new(TILE, TILE);
pub const TILE_TEXELS: f32 = 18.0;
pub const CHARACTER_TEXELS: f32 = 24.0;
pub const CHARACTER_TILES: f32 = CHARACTER_TEXELS / TILE_TEXELS;
pub const PIXELS_PER_UNIT: f32 = 72.0;

pub const CRATE: u32 = 9;
pub const COIN: u32 = 151;
pub const FLAG: u32 = 111;
pub const PLAYER_FRAMES: [u32; 2] = [0, 1];

pub const LAYER_BACKDROP: i32 = -30;
pub const LAYER_SKY: i32 = -20;
pub const LAYER_PROPS: i32 = -5;
pub const LAYER_TILES: i32 = 0;
pub const LAYER_PICKUPS: i32 = 1;
pub const LAYER_ACTORS: i32 = 2;

pub const SKY_TILE: f32 = CHARACTER_TEXELS / TILE_TEXELS;
pub const SKY_ROWS: u32 = 9;
pub const SKY_HORIZON: u32 = 3;
pub const SKY_ORIGIN: f32 = -0.34;
pub const SKY_PARALLAX: f32 = 0.62;
pub const SKY_MARGIN: f32 = 18.0;

pub const HAZE_SPAN: f32 = 600.0;
pub const BACKDROP_DEPTH: f32 = -5.0;
pub const SKY: [f32; 4] = [0.53, 0.81, 0.86, 1.0];

pub const RUN_SPEED: f32 = 8.0;
pub const GROUND_ACCELERATION: f32 = 54.0;
pub const TURN_ACCELERATION: f32 = 108.0;
pub const AIR_ACCELERATION: f32 = 34.0;
pub const GROUND_FRICTION: f32 = 44.0;
pub const AIR_FRICTION: f32 = 6.0;
pub const RISE_GRAVITY: f32 = 66.0;
pub const FALL_GRAVITY: f32 = 106.0;
pub const JUMP_SPEED: f32 = 23.0;
pub const SHORT_JUMP_SPEED: f32 = 11.5;
pub const TERMINAL_SPEED: f32 = 24.0;
pub const COYOTE_SECONDS: f32 = 0.1;
pub const BUFFER_SECONDS: f32 = 0.1;
pub const STOMP_SPEED: f32 = 15.0;
pub const STOMP_HELD_SPEED: f32 = 21.0;
pub const MAX_STEP: f32 = 0.033;
pub const SKIN: f32 = 0.002;

pub const PLAYER_HALF_WIDTH: f32 = 0.3;
pub const PLAYER_HEIGHT: f32 = 1.1;
pub const WALK_FRAME_SECONDS: f32 = 0.12;

pub const WALKER_SPEED: f32 = 2.2;
pub const WALKER_HALF_WIDTH: f32 = 0.36;
pub const WALKER_HEIGHT: f32 = 0.7;
pub const WALKER_FRAME_SECONDS: f32 = 0.18;
pub const STOMP_MARGIN: f32 = 0.35;
pub const HIT_SECONDS: f32 = 3.0;
pub const HIT_KNOCKBACK: f32 = 6.0;
pub const HIT_LIFT: f32 = 7.0;

pub const COIN_SECONDS: f32 = 2.0;
pub const STOMP_COINS: u32 = 3;
pub const COIN_KIND: u32 = 0;
pub const COIN_GRASP: f32 = 1.0;
pub const COIN_BOB: f32 = 0.14;
pub const COIN_BOB_SPEED: f32 = 3.4;
pub const FLAG_REACH: f32 = 0.5;
pub const FALL_LIMIT: f32 = -3.0;
pub const GROUND_HEIGHT: f32 = 4.0;

pub const CAMERA_FOLLOW: f32 = 7.0;
pub const CAMERA_LEAD: f32 = 0.18;
pub const CAMERA_RISE: f32 = 2.4;
pub const CAMERA_DEPTH: f32 = 20.0;
pub const ATTRACT_SPEED: f32 = 3.2;
pub const ATTRACT_START: f32 = 20.0;
pub const ATTRACT_RISE: f32 = 2.8;

pub const READOUT_WIDTH: f32 = 470.0;
pub const READOUT_HEIGHT: f32 = 46.0;
pub const READOUT_MARGIN: f32 = 16.0;
pub const READOUT_SIZE: f32 = 24.0;

pub const TITLE_WIDTH: f32 = 500.0;
pub const TITLE_HEIGHT: f32 = 314.0;
pub const MENU_WIDTH: f32 = 400.0;
pub const MENU_HEIGHT: f32 = 300.0;
pub const MAP_WIDTH: f32 = 520.0;
pub const MAP_HEIGHT: f32 = 340.0;
pub const TITLE_SIZE: f32 = 66.0;
pub const HEADING_SIZE: f32 = 34.0;
pub const SUBTITLE_SIZE: f32 = 22.0;
pub const LABEL_SIZE: f32 = 20.0;
pub const ROW_GAP: f32 = 10.0;
pub const ROW_PADDING: f32 = 20.0;
pub const RULE_HEIGHT: f32 = 4.0;
pub const RULE_WIDTH: f32 = 360.0;
pub const DOT_SIZE: f32 = 40.0;
pub const DOT_GAP: f32 = 16.0;

pub const CLEAR: Vec4 = Vec4::new(0.0, 0.0, 0.0, 0.0);
pub const VEIL: Vec4 = Vec4::new(0.03, 0.04, 0.08, 0.58);
pub const PANEL_BACKGROUND: Vec4 = Vec4::new(0.05, 0.05, 0.09, 0.88);
pub const PANEL_BORDER: Vec4 = Vec4::new(1.0, 0.84, 0.29, 0.38);
pub const TEXT_PRIMARY: Vec4 = Vec4::new(1.0, 1.0, 1.0, 1.0);
pub const TEXT_GOLD: Vec4 = Vec4::new(1.0, 0.84, 0.29, 1.0);
pub const TEXT_DIM: Vec4 = Vec4::new(0.80, 0.83, 0.90, 1.0);
pub const TEXT_URGENT: Vec4 = Vec4::new(1.0, 0.42, 0.36, 1.0);
pub const TEXT_OUTLINE: Vec4 = Vec4::new(0.0, 0.0, 0.0, 1.0);
pub const DOT_SELECTED: Vec4 = Vec4::new(1.0, 0.84, 0.29, 1.0);
pub const DOT_OPEN: Vec4 = Vec4::new(0.45, 0.70, 0.45, 1.0);
pub const DOT_LOCKED: Vec4 = Vec4::new(0.22, 0.23, 0.28, 1.0);
pub const URGENT_SECONDS: f32 = 6.0;
