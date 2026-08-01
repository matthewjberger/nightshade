use nalgebra_glm::Vec3;

pub const TITLE: &str = "Toast card";
pub const REACH: f32 = 12.0;
pub const FLOOR_TINT: [f32; 4] = [0.15, 0.15, 0.18, 1.0];
pub const BODY_TINT: [f32; 4] = [0.74, 0.70, 0.62, 1.0];
pub const SIGN_TINT: [f32; 4] = [0.85, 0.72, 0.45, 1.0];
pub const SHOWING: f32 = 2.2;
pub const FADING: f32 = 0.6;
pub const SIGN_LOW: f32 = 0.1;
pub const SIGN_HIGH: f32 = 2.2;
pub const SIGN_BACK: f32 = 3.0;
pub const LINES: [&str; 4] = [
    "the gate is open",
    "a crate arrived",
    "the pump is dry",
    "the shift is over",
];

pub const CAMERA_TARGET: Vec3 = Vec3::new(0.0, 1.2, 0.0);
pub const CAMERA_BACK: f32 = 16.0;
pub const CAMERA_YAW: f32 = 0.2;
pub const CAMERA_PITCH: f32 = 0.35;
