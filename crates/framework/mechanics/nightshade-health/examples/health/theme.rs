use nalgebra_glm::Vec3;

pub const TITLE: &str = "Health";
pub const REACH: f32 = 14.0;
pub const FLOOR_TINT: [f32; 4] = [0.15, 0.15, 0.18, 1.0];
pub const BODY_TINT: [f32; 4] = [0.74, 0.70, 0.62, 1.0];
pub const BODIES: usize = 5;
pub const SPACING: f32 = 3.0;
pub const FULL: f32 = 100.0;
pub const BLOW: f32 = 18.0;
pub const MEND: f32 = 12.0;
pub const STEP_OUT: f32 = 1.8;

pub const CAMERA_TARGET: Vec3 = Vec3::new(0.0, 1.0, 0.0);
pub const CAMERA_BACK: f32 = 22.0;
pub const CAMERA_YAW: f32 = 0.0;
pub const CAMERA_PITCH: f32 = 0.35;
