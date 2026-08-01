use nalgebra_glm::Vec3;

pub const TITLE: &str = "Pen";
pub const REACH: f32 = 14.0;
pub const FLOOR_TINT: [f32; 4] = [0.15, 0.15, 0.18, 1.0];
pub const WALL_TINT: [f32; 4] = [0.30, 0.32, 0.38, 1.0];
pub const WALKER_TINT: [f32; 4] = [0.92, 0.42, 0.46, 1.0];
pub const FREE_TINT: [f32; 4] = [0.42, 0.78, 0.92, 1.0];
pub const ARENA: f32 = 5.0;
pub const SHOVE: f32 = 7.0;

pub const CAMERA_TARGET: Vec3 = Vec3::new(0.0, 1.0, 0.0);
pub const CAMERA_BACK: f32 = 26.0;
pub const CAMERA_YAW: f32 = 0.5;
pub const CAMERA_PITCH: f32 = 0.7;
