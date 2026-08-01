use nalgebra_glm::Vec3;

pub const TITLE: &str = "Dialogue card";
pub const REACH: f32 = 12.0;
pub const FLOOR_TINT: [f32; 4] = [0.15, 0.15, 0.18, 1.0];
pub const BODY_TINT: [f32; 4] = [0.74, 0.70, 0.62, 1.0];
pub const LETTERS_A_SECOND: f32 = 28.0;
pub const SPEAKER: &str = "Warden";
pub const LINES: [&str; 3] = [
    "The gate holds for now.",
    "Bring back what the river took.",
    "Then we will speak of payment.",
];

pub const CAMERA_TARGET: Vec3 = Vec3::new(0.0, 1.2, 0.0);
pub const CAMERA_BACK: f32 = 16.0;
pub const CAMERA_YAW: f32 = 0.2;
pub const CAMERA_PITCH: f32 = 0.35;
