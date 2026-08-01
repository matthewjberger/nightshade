use nalgebra_glm::Vec3;

pub const TITLE: &str = "Hover";
pub const REACH: f32 = 12.0;
pub const FLOOR_TINT: [f32; 4] = [0.15, 0.15, 0.18, 1.0];
pub const COIN_TINT: [f32; 4] = [0.95, 0.80, 0.25, 1.0];
pub const COINS: usize = 7;
pub const SPACING: f32 = 2.2;
pub const PERCH: f32 = 1.6;
pub const RATE: f32 = 1.6;
pub const PHASE: f32 = 0.6;
pub const SPIN: f32 = 2.4;
pub const RISE: f32 = 3.0;
pub const PERCH_LOW: f32 = 0.6;
pub const PERCH_HIGH: f32 = 6.0;
pub const BOB_STEP: f32 = 0.1;
pub const BOB_MOST: f32 = 1.6;
pub const RATE_STEP: f32 = 0.3;
pub const RATE_HIGH: f32 = 6.0;
pub const STEP_OUT: f32 = 2.0;

pub const CAMERA_TARGET: Vec3 = Vec3::new(0.0, 1.4, 0.0);
pub const CAMERA_BACK: f32 = 20.0;
pub const CAMERA_YAW: f32 = 0.2;
pub const CAMERA_PITCH: f32 = 0.25;
