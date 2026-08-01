use nalgebra_glm::Vec3;

pub const TITLE: &str = "Shake";
pub const REACH: f32 = 12.0;
pub const FLOOR_TINT: [f32; 4] = [0.15, 0.15, 0.18, 1.0];
pub const POST_TINT: [f32; 4] = [0.30, 0.32, 0.38, 1.0];
pub const HAMMER_TINT: [f32; 4] = [0.92, 0.42, 0.46, 1.0];
pub const POSTS: usize = 6;
pub const SPACING: f32 = 2.4;
pub const HAMMER_HEIGHT: f32 = 3.4;
pub const BLOW: f32 = 0.7;
pub const DECAY: f32 = 1.2;
pub const SHAKE_REACH: f32 = 0.5;
pub const TURN: f32 = 0.12;

pub const CAMERA_TARGET: Vec3 = Vec3::new(0.0, 1.2, 0.0);
pub const CAMERA_BACK: f32 = 20.0;
pub const CAMERA_YAW: f32 = 0.3;
pub const CAMERA_PITCH: f32 = 0.35;
