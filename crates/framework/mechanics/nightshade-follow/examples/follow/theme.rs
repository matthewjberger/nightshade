use nalgebra_glm::Vec3;

pub const TITLE: &str = "Follow";
pub const REACH: f32 = 14.0;
pub const FLOOR_TINT: [f32; 4] = [0.15, 0.15, 0.18, 1.0];
pub const LEAD_TINT: [f32; 4] = [0.92, 0.70, 0.36, 1.0];
pub const TAIL_TINT: [f32; 4] = [0.42, 0.78, 0.92, 1.0];
pub const TAIL: usize = 6;
pub const RING: f32 = 7.0;
pub const FOLLOW_RATE: f32 = 4.0;
pub const GAP: f32 = 1.4;
pub const WALK: f32 = 7.0;
pub const RATE_STEP: f32 = 2.0;
pub const SLOWEST: f32 = 0.5;
pub const FASTEST: f32 = 20.0;
pub const SNAP: f32 = 0.5;

pub const CAMERA_TARGET: Vec3 = Vec3::new(0.0, 1.0, 0.0);
pub const CAMERA_BACK: f32 = 26.0;
pub const CAMERA_YAW: f32 = 0.4;
pub const CAMERA_PITCH: f32 = 0.6;
