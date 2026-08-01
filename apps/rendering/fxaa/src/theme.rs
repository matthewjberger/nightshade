pub const TITLE: &str = "FXAA";

pub const SETTLED: u64 = 90;

pub const CAMERA_FOCUS: [f32; 3] = [-1.5, 1.7, 0.0];
pub const CAMERA_BACK: f32 = 6.4;
pub const CAMERA_YAW: f32 = 0.0;
pub const CAMERA_PITCH: f32 = 0.02;

pub const BARS: usize = 15;
pub const BAR_LENGTH: f32 = 3.4;
pub const BAR_THICKNESS: f32 = 0.04;
pub const BAR_HEIGHT: f32 = 1.7;
pub const BAR_COLOR: [f32; 3] = [1.0, 1.0, 1.0];

pub const BLOCKS: [(f32, f32); 2] = [(-2.9, 0.5), (2.9, -0.4)];
pub const BLOCK_REACH: f32 = 1.1;
pub const BLOCK_BACK: f32 = -3.2;
pub const BLOCK_COLOR: [f32; 4] = [0.72, 0.74, 0.78, 1.0];
pub const BLOCK_ROUGHNESS: f32 = 0.55;

pub const FLOOR_REACH: f32 = 18.0;
pub const FLOOR_THICKNESS: f32 = 0.05;
pub const FLOOR_COLOR: [f32; 4] = [0.035, 0.037, 0.042, 1.0];
pub const FLOOR_ROUGHNESS: f32 = 0.85;

pub const PIXELS_PER_STEP: f32 = 3.0;
pub const PIXELS_LEAST: f32 = 1.0;
pub const PIXELS_MOST: f32 = 4.0;

pub const SWITCH_TEST_ID: &str = "smoothing";
pub const TOGGLE_FRAME: u64 = 40;
pub const TOGGLE_SETTLE: u64 = 4;

pub const PANEL_WIDTH: f32 = 268.0;
pub const PANEL_PADDING: f32 = 12.0;
pub const PANEL_SPACING: f32 = 8.0;
