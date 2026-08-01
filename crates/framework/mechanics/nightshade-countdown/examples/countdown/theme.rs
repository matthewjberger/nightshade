use nalgebra_glm::Vec3;

pub const TITLE: &str = "Countdown";
pub const REACH: f32 = 12.0;
pub const FLOOR_TINT: [f32; 4] = [0.15, 0.15, 0.18, 1.0];
pub const TRACK_TINT: [f32; 4] = [0.10, 0.10, 0.12, 1.0];
pub const BAR_TINT: [f32; 4] = [0.92, 0.42, 0.46, 1.0];
pub const SECONDS: f32 = 6.0;
pub const BAR_WIDTH: f32 = 12.0;
pub const PENALTY: f32 = 1.0;

pub const CAMERA_TARGET: Vec3 = Vec3::new(0.0, 2.0, 0.0);
pub const CAMERA_BACK: f32 = 20.0;
pub const CAMERA_YAW: f32 = 0.0;
pub const CAMERA_PITCH: f32 = 0.25;
