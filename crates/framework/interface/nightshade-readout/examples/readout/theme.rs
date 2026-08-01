use nalgebra_glm::Vec3;

pub const TITLE: &str = "Readout";
pub const REACH: f32 = 12.0;
pub const FLOOR_TINT: [f32; 4] = [0.15, 0.15, 0.18, 1.0];
pub const RUNNER_TINT: [f32; 4] = [0.42, 0.78, 0.92, 1.0];
pub const POSTS: usize = 5;
pub const ROW_GAP: f32 = 2.4;
pub const POST_LOW: f32 = 0.3;
pub const POST_HIGH: f32 = 2.6;
pub const POST_WIDE: f32 = 1.0;
pub const LINES: [&str; POSTS] = [
    "a wide floor",
    "one sun overhead",
    "five posts in a row",
    "an orbiting camera",
    "a title above it all",
];

pub const CAMERA_TARGET: Vec3 = Vec3::new(0.0, 1.0, 0.0);
pub const CAMERA_BACK: f32 = 18.0;
pub const CAMERA_YAW: f32 = 0.3;
pub const CAMERA_PITCH: f32 = 0.5;
