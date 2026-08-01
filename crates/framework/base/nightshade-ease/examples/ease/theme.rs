use nalgebra_glm::Vec3;

pub const TITLE: &str = "Ease";
pub const REACH: f32 = 14.0;
pub const FLOOR_TINT: [f32; 4] = [0.15, 0.15, 0.18, 1.0];
pub const RUNNER_TINT: [f32; 4] = [0.42, 0.78, 0.92, 1.0];
pub const MARK_TINT: [f32; 4] = [0.92, 0.70, 0.36, 1.0];
pub const RUNNERS: usize = 5;
pub const SPACING: f32 = 2.4;
pub const FROM: f32 = -10.0;
pub const TO: f32 = 10.0;
pub const SCRUB: f32 = 0.45;
pub const RATE: f32 = 2.2;
pub const STEP: f32 = 7.0;

pub const CAMERA_TARGET: Vec3 = Vec3::new(0.0, 1.0, 0.0);
pub const CAMERA_BACK: f32 = 26.0;
pub const CAMERA_YAW: f32 = 0.0;
pub const CAMERA_PITCH: f32 = 0.55;
