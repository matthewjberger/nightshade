use nalgebra_glm::Vec3;

pub const TITLE: &str = "Noise";
pub const COLUMNS: i32 = 40;
pub const STEP: f32 = 1.0;
pub const LIFT: f32 = 7.0;
pub const SCALE: f32 = 0.09;
pub const LAYERS: u32 = 5;
pub const MOST_LAYERS: u32 = 8;
pub const FADE: f32 = 0.5;
pub const SEED: u32 = 9;

pub const LOW_TINT: [f32; 4] = [0.18, 0.30, 0.42, 1.0];
pub const HIGH_TINT: [f32; 4] = [0.78, 0.74, 0.58, 1.0];
pub const WALKER_TINT: [f32; 4] = [0.90, 0.36, 0.34, 1.0];
pub const WALKER_LIFT: f32 = 0.6;
pub const WALK_RATE: f32 = 8.0;

pub const CAMERA_TARGET: Vec3 = Vec3::new(0.0, 2.0, 0.0);
pub const CAMERA_BACK: f32 = 46.0;
pub const CAMERA_YAW: f32 = 0.7;
pub const CAMERA_PITCH: f32 = 0.55;
