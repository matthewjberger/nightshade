use nalgebra_glm::{Vec3, Vec4};

pub const TITLE: &str = "Signed Distance Fields";
pub const SETTLED: u64 = 120;

pub const CAMERA_START: Vec3 = Vec3::new(0.0, 22.0, 40.0);
pub const CAMERA_YAW: f32 = 0.0;
pub const CAMERA_PITCH: f32 = -0.28;

pub const GROUND_ORDER: u32 = 0;
pub const FIRST_TOOL_ORDER: u32 = 1;
pub const BODY_BAND: u32 = 1 << 24;
pub const HOLE_ORDER: u32 = BODY_BAND;
pub const BLOB_ORDER: u32 = BODY_BAND + 10;
pub const CUBE_ORDER: u32 = BODY_BAND + 11;
pub const RING_ORDER: u32 = BODY_BAND + 12;
pub const RING_SEGMENTS: u32 = 48;
pub const RING_SIDES: u32 = 16;
pub const PROP_ORDER: u32 = BODY_BAND + 13;
pub const TUNNEL_ORDER: u32 = BODY_BAND + 21;

pub const STONE_START: Vec3 = Vec3::new(0.0, 13.0, 18.0);
pub const STONE_LIMIT: usize = 24;

pub const AIM_STEPS: u32 = 256;
pub const AIM_TOUCH: f32 = 0.02;
pub const AIM_REACH: f32 = 200.0;

pub const MARCH_SCALES: [f32; 3] = [0.5, 0.75, 1.0];

pub const MARGIN: f32 = 16.0;
pub const PANEL_WIDTH: f32 = 262.0;
pub const PANEL_HEIGHT: f32 = 70.0;
pub const PANEL_BACKGROUND: Vec4 = Vec4::new(0.05, 0.05, 0.08, 0.78);
pub const PANEL_BORDER: Vec4 = Vec4::new(1.0, 1.0, 1.0, 0.12);
pub const TEXT_DIM: Vec4 = Vec4::new(0.78, 0.80, 0.86, 1.0);
pub const TEXT_BRIGHT: Vec4 = Vec4::new(1.0, 1.0, 1.0, 1.0);

pub const BAR_BOTTOM: f32 = 97.0;
pub const SLOT_WIDTH: f32 = 112.0;
pub const SLOT_HEIGHT: f32 = 40.0;
pub const SLOT_GAP: f32 = 8.0;
pub const SLOT_IDLE: Vec4 = Vec4::new(1.0, 1.0, 1.0, 0.04);
pub const SLOT_EQUIPPED: Vec4 = Vec4::new(0.30, 0.56, 1.0, 0.55);

pub const RATE_BANDS: [(f32, Vec4); 3] = [
    (50.0, Vec4::new(0.45, 0.92, 0.50, 1.0)),
    (25.0, Vec4::new(1.0, 0.68, 0.25, 1.0)),
    (0.0, Vec4::new(1.0, 0.36, 0.36, 1.0)),
];
