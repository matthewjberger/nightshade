use nalgebra_glm::Vec3;

pub const TITLE: &str = "Cooldown";
pub const REACH: f32 = 12.0;
pub const FLOOR_TINT: [f32; 4] = [0.15, 0.15, 0.18, 1.0];
pub const READY_TINT: [f32; 4] = [0.42, 0.78, 0.92, 1.0];
pub const SPACING: f32 = 2.6;
pub const GUNS: usize = 5;
pub const SHORTEST: f32 = 0.6;
pub const STEP: f32 = 0.7;
pub const LIFT: f32 = 2.2;
pub const SNAP: f32 = 0.25;

pub const CAMERA_TARGET: Vec3 = Vec3::new(0.0, 1.2, 0.0);
pub const CAMERA_BACK: f32 = 20.0;
pub const CAMERA_YAW: f32 = 0.2;
pub const CAMERA_PITCH: f32 = 0.4;
