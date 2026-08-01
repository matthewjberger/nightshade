use nalgebra_glm::Vec3;

pub const TITLE: &str = "Wander";
pub const REACH: f32 = 16.0;
pub const FLOOR_TINT: [f32; 4] = [0.15, 0.15, 0.18, 1.0];
pub const WALKER_TINT: [f32; 4] = [0.52, 0.86, 0.58, 1.0];
pub const HOME_TINT: [f32; 4] = [0.30, 0.32, 0.38, 1.0];
pub const WALKERS: usize = 9;
pub const SPREAD: f32 = 10.0;
pub const ROAM: f32 = 4.0;
pub const SPEED: f32 = 2.2;
pub const ARRIVAL: f32 = 0.4;
pub const REST_LOW: f32 = 0.6;
pub const REST_HIGH: f32 = 2.4;
pub const PACE_STEP: f32 = 0.25;
pub const PACE_MOST: f32 = 3.0;

pub const CAMERA_TARGET: Vec3 = Vec3::new(0.0, 1.0, 0.0);
pub const CAMERA_BACK: f32 = 30.0;
pub const CAMERA_YAW: f32 = 0.5;
pub const CAMERA_PITCH: f32 = 0.7;
