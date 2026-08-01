use nalgebra_glm::Vec3;

pub const TITLE: &str = "Day and night";
pub const REACH: f32 = 14.0;
pub const FLOOR_TINT: [f32; 4] = [0.15, 0.15, 0.18, 1.0];
pub const POST_TINT: [f32; 4] = [0.62, 0.66, 0.74, 1.0];
pub const POSTS: usize = 8;
pub const RING: f32 = 6.0;
pub const HOURS_A_SECOND: f32 = 1.5;
pub const DAWN: f32 = 6.0;
pub const DUSK: f32 = 19.0;
pub const START_HOUR: f32 = 5.0;
pub const SCRUB: f32 = 8.0;
pub const SUN_BACK: f32 = 20.0;
pub const DAY_SKY: [f32; 4] = [0.45, 0.60, 0.80, 1.0];
pub const NIGHT_SKY: [f32; 4] = [0.03, 0.04, 0.09, 1.0];

pub const CAMERA_TARGET: Vec3 = Vec3::new(0.0, 1.5, 0.0);
pub const CAMERA_BACK: f32 = 24.0;
pub const CAMERA_YAW: f32 = 0.4;
pub const CAMERA_PITCH: f32 = 0.4;
