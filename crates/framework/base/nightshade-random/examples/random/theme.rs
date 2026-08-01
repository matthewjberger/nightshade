use nalgebra_glm::Vec3;

pub const TITLE: &str = "Random";
pub const REACH: f32 = 14.0;
pub const FLOOR_TINT: [f32; 4] = [0.15, 0.15, 0.18, 1.0];
pub const DISC_TINT: [f32; 4] = [0.42, 0.78, 0.92, 1.0];
pub const RING_TINT: [f32; 4] = [0.92, 0.70, 0.36, 1.0];
pub const GRAINS: usize = 64;
pub const SPREAD: f32 = 9.0;
pub const NUDGE: f32 = 1.0;
pub const NARROWEST: f32 = 2.0;
pub const WIDEST: f32 = 13.0;
pub const SEED: u64 = 0x5eed;

pub const CAMERA_TARGET: Vec3 = Vec3::new(0.0, 1.0, 0.0);
pub const CAMERA_BACK: f32 = 30.0;
pub const CAMERA_YAW: f32 = 0.5;
pub const CAMERA_PITCH: f32 = 0.7;
