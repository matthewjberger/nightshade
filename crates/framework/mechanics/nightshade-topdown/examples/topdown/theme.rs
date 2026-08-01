use nalgebra_glm::Vec3;

pub const TITLE: &str = "Top Down";
pub const REACH: f32 = 18.0;
pub const FLOOR_TINT: [f32; 4] = [0.17, 0.19, 0.17, 1.0];
pub const WALKER_TINT: [f32; 4] = [0.88, 0.78, 0.40, 1.0];
pub const POST_TINT: [f32; 4] = [0.40, 0.42, 0.48, 1.0];
pub const SPEED: f32 = 6.0;
pub const STROLLING: f32 = 1.0;
pub const RUNNING: f32 = 2.4;

pub const CAMERA_TARGET: Vec3 = Vec3::new(0.0, 0.0, 0.0);
pub const CAMERA_BACK: f32 = 30.0;
pub const CAMERA_YAW: f32 = 0.0;
pub const CAMERA_PITCH: f32 = 1.1;
