use nalgebra_glm::Vec3;

pub const TITLE: &str = "Clock";
pub const REACH: f32 = 12.0;
pub const FLOOR_TINT: [f32; 4] = [0.15, 0.15, 0.18, 1.0];
pub const MARK_TINT: [f32; 4] = [0.30, 0.32, 0.38, 1.0];
pub const HAND_TINT: [f32; 4] = [0.92, 0.70, 0.36, 1.0];
pub const HOURS: usize = 24;
pub const DIAL: f32 = 7.0;
pub const HOURS_A_SECOND: f32 = 4.0;
pub const DAWN: f32 = 6.0;
pub const NOON: f32 = 12.0;
pub const DUSK: f32 = 18.0;
pub const MIDNIGHT: f32 = 0.0;
pub const DAY_FROM: f32 = 6.0;
pub const DAY_TO: f32 = 18.0;

pub const CAMERA_TARGET: Vec3 = Vec3::new(0.0, 0.5, 0.0);
pub const CAMERA_BACK: f32 = 22.0;
pub const CAMERA_YAW: f32 = 0.0;
pub const CAMERA_PITCH: f32 = 0.9;
