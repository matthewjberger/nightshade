use nalgebra_glm::Vec3;

pub const TITLE: &str = "Dialogue";
pub const REACH: f32 = 10.0;
pub const FLOOR_TINT: [f32; 4] = [0.14, 0.13, 0.17, 1.0];
pub const SPEAKER_TINT: [f32; 4] = [0.80, 0.70, 0.90, 1.0];
pub const LETTERS_A_SECOND: f32 = 28.0;
pub const LEAN: f32 = 0.14;
pub const STEP: f32 = 1.6;

pub const SPEAKER: &str = "Warden";
pub const LINES: [&str; 4] = [
    "You came a long way for a locked door.",
    "The key went into the well two winters ago.",
    "I would not go down after it.",
    "But you look like someone who would.",
];
pub const REPLIES: [&str; 2] = ["I will go down.", "Keep your door."];

pub const CAMERA_TARGET: Vec3 = Vec3::new(0.0, 1.0, 0.0);
pub const CAMERA_BACK: f32 = 9.0;
pub const CAMERA_YAW: f32 = 0.4;
pub const CAMERA_PITCH: f32 = 0.25;
