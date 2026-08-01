use nalgebra_glm::{Vec3, Vec4};

pub const TITLE: &str = "Flash";
pub const REACH: f32 = 14.0;
pub const FLOOR_TINT: [f32; 4] = [0.20, 0.20, 0.23, 1.0];
pub const BLOCK_TINT: [f32; 4] = [0.55, 0.50, 0.45, 1.0];
pub const BLOCKS: usize = 5;
pub const SPACING: f32 = 3.0;

pub const HURT: Vec4 = Vec4::new(0.55, 0.02, 0.0, 0.55);
pub const SHIFT: Vec4 = Vec4::new(0.9, 0.85, 0.8, 0.9);
pub const MUZZLE: Vec4 = Vec4::new(1.0, 0.9, 0.6, 0.35);
pub const HURT_FADE: f32 = 2.2;
pub const SHIFT_FADE: f32 = 0.6;
pub const MUZZLE_FADE: f32 = 9.0;
pub const NAMES: [&str; 3] = ["hurt", "shift", "muzzle"];

pub const CAMERA_TARGET: Vec3 = Vec3::new(0.0, 1.2, 0.0);
pub const CAMERA_BACK: f32 = 16.0;
pub const CAMERA_YAW: f32 = 0.4;
pub const CAMERA_PITCH: f32 = 0.28;
