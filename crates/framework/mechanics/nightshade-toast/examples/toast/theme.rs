use nalgebra_glm::Vec3;

pub const TITLE: &str = "Toast";
pub const REACH: f32 = 12.0;
pub const FLOOR_TINT: [f32; 4] = [0.15, 0.15, 0.18, 1.0];
pub const CARD_TINT: [f32; 4] = [0.92, 0.70, 0.36, 1.0];
pub const SHOWING: f32 = 2.2;
pub const FADING: f32 = 0.8;
pub const LINES: [&str; 4] = [
    "a door opened",
    "the pump started",
    "a crate arrived",
    "the shift ended",
];

pub const CAMERA_TARGET: Vec3 = Vec3::new(0.0, 1.5, 0.0);
pub const CAMERA_BACK: f32 = 16.0;
pub const CAMERA_YAW: f32 = 0.0;
pub const CAMERA_PITCH: f32 = 0.3;
