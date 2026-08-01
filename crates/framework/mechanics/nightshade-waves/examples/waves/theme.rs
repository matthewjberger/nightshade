use nalgebra_glm::Vec3;

pub const TITLE: &str = "Waves";
pub const REACH: f32 = 24.0;
pub const FLOOR_TINT: [f32; 4] = [0.12, 0.14, 0.18, 1.0];
pub const KIND_TINT: [[f32; 4]; 3] = [
    [0.86, 0.46, 0.34, 1.0],
    [0.44, 0.78, 0.52, 1.0],
    [0.46, 0.58, 0.90, 1.0],
];
pub const LANE: f32 = 16.0;
pub const WALK: f32 = 2.6;

pub const CAMERA_TARGET: Vec3 = Vec3::new(0.0, 0.0, 0.0);
pub const CAMERA_BACK: f32 = 38.0;
pub const CAMERA_YAW: f32 = 0.5;
pub const CAMERA_PITCH: f32 = 0.9;
