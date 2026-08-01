use nalgebra_glm::Vec3;

pub const TITLE: &str = "Plane";
pub const REACH: f32 = 12.0;
pub const FLOOR_TINT: [f32; 4] = [0.15, 0.15, 0.18, 1.0];
pub const GROUND_TINT: [f32; 4] = [0.42, 0.78, 0.92, 1.0];
pub const SIDE_TINT: [f32; 4] = [0.92, 0.70, 0.36, 1.0];
pub const MIDDLE_TINT: [f32; 4] = [0.82, 0.82, 0.86, 1.0];
pub const RING: f32 = 8.0;
pub const RATE: f32 = 6.0;

pub const CAMERA_TARGET: Vec3 = Vec3::new(0.0, 3.0, 0.0);
pub const CAMERA_BACK: f32 = 26.0;
pub const CAMERA_YAW: f32 = 0.9;
pub const CAMERA_PITCH: f32 = 0.35;
