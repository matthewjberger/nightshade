use nalgebra_glm::Vec3;

pub const TITLE: &str = "Flashlight";
pub const REACH: f32 = 18.0;
pub const FLOOR_TINT: [f32; 4] = [0.14, 0.14, 0.15, 1.0];
pub const CRATE_TINT: [f32; 4] = [0.45, 0.38, 0.30, 1.0];
pub const CRATES: usize = 9;
pub const RING: f32 = 6.0;
pub const TORCH_HEIGHT: f32 = 2.2;
pub const TORCH_COLOR: Vec3 = Vec3::new(1.0, 0.93, 0.8);
pub const BEAM: f32 = 210.0;
pub const RANGE: f32 = 30.0;
pub const INNER: f32 = 0.20;
pub const OUTER: f32 = 0.46;
pub const FULL: f32 = 100.0;
pub const LOW: f32 = 22.0;
pub const BROWNOUT: f32 = 0.45;
pub const DRAIN: f32 = 6.0;
pub const CHARGE: f32 = 55.0;
pub const SWEEP: f32 = 0.5;

pub const CAMERA_TARGET: Vec3 = Vec3::new(0.0, 1.6, 0.0);
pub const CAMERA_BACK: f32 = 20.0;
pub const CAMERA_YAW: f32 = 0.6;
pub const CAMERA_PITCH: f32 = 0.30;
