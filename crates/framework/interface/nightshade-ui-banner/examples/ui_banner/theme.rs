use nalgebra_glm::{Vec3, Vec4};

pub const TITLE: &str = "Banner";
pub const REACH: f32 = 12.0;
pub const FLOOR_TINT: [f32; 4] = [0.15, 0.15, 0.18, 1.0];
pub const BODY_TINT: [f32; 4] = [0.74, 0.70, 0.62, 1.0];
pub const CALM: Vec4 = Vec4::new(0.92, 0.94, 1.0, 1.0);
pub const URGENT: Vec4 = Vec4::new(0.95, 0.45, 0.42, 1.0);
pub const LINES: [&str; 4] = [
    "Round 1   0/6   Time 30",
    "Round 1   4/6   Time 21",
    "Round 2   1/8   Time 30",
    "Round 2   7/8   Time 4",
];

pub const CAMERA_TARGET: Vec3 = Vec3::new(0.0, 1.2, 0.0);
pub const CAMERA_BACK: f32 = 16.0;
pub const CAMERA_YAW: f32 = 0.2;
pub const CAMERA_PITCH: f32 = 0.35;
