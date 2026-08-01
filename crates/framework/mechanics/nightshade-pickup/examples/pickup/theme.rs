use nalgebra_glm::Vec3;

pub const TITLE: &str = "Pickup";
pub const REACH: f32 = 18.0;
pub const FLOOR_TINT: [f32; 4] = [0.13, 0.16, 0.20, 1.0];
pub const COIN_TINT: [f32; 4] = [0.94, 0.80, 0.30, 1.0];
pub const COLLECTOR_TINT: [f32; 4] = [0.40, 0.80, 0.62, 1.0];
pub const COINS: usize = 24;
pub const RING: f32 = 10.0;
pub const WALK: f32 = 7.0;
pub const GRASP: f32 = 1.6;
pub const COIN_KIND: u32 = 1;

pub const CAMERA_TARGET: Vec3 = Vec3::new(0.0, 0.0, 0.0);
pub const CAMERA_BACK: f32 = 30.0;
pub const CAMERA_YAW: f32 = 0.4;
pub const CAMERA_PITCH: f32 = 1.0;
