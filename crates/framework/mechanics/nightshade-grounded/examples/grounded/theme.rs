use nalgebra_glm::Vec3;

pub const TITLE: &str = "Grounded";
pub const WALKER_TINT: [f32; 4] = [0.92, 0.70, 0.36, 1.0];
pub const HILL_TINT: [f32; 4] = [0.28, 0.42, 0.34, 1.0];
pub const COLUMNS: i32 = 24;
pub const STEP: f32 = 1.2;
pub const LIFT: f32 = 3.0;
pub const WAVE: f32 = 0.45;
pub const EDGE: f32 = 12.0;
pub const STAND: f32 = 0.6;
pub const STAND_STEP: f32 = 0.2;
pub const STAND_LOW: f32 = 0.0;
pub const STAND_HIGH: f32 = 4.0;
pub const WALK: f32 = 7.0;

pub const CAMERA_TARGET: Vec3 = Vec3::new(0.0, 2.0, 0.0);
pub const CAMERA_BACK: f32 = 34.0;
pub const CAMERA_YAW: f32 = 0.7;
pub const CAMERA_PITCH: f32 = 0.45;
