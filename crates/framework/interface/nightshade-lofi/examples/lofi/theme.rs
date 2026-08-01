use nalgebra_glm::Vec3;

pub const TITLE: &str = "Lofi";
pub const REACH: f32 = 14.0;
pub const FLOOR_TINT: [f32; 4] = [0.18, 0.17, 0.20, 1.0];
pub const PILLAR_TINT: [f32; 4] = [0.62, 0.48, 0.38, 1.0];
pub const BALL_TINT: [f32; 4] = [0.35, 0.62, 0.68, 1.0];
pub const PILLARS: usize = 7;
pub const SPACING: f32 = 3.0;
pub const PILLAR_HEIGHT: f32 = 3.2;
pub const TURN_RATE: f32 = 0.6;

pub const CAMERA_TARGET: Vec3 = Vec3::new(0.0, 1.6, 0.0);
pub const CAMERA_BACK: f32 = 16.0;
pub const CAMERA_YAW: f32 = 0.5;
pub const CAMERA_PITCH: f32 = 0.25;
