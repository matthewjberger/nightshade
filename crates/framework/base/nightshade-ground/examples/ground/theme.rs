use nalgebra_glm::Vec3;

pub const TITLE: &str = "Ground";
pub const PROBE_TINT: [f32; 4] = [0.92, 0.42, 0.46, 1.0];
pub const RESTING_TINT: [f32; 4] = [0.86, 0.72, 0.40, 1.0];
pub const BURIED_TINT: [f32; 4] = [0.36, 0.86, 0.62, 1.0];
pub const HOVER: f32 = 0.25;
pub const HILL_TINT: [f32; 4] = [0.30, 0.44, 0.36, 1.0];
pub const SOLID_TINT: [f32; 4] = [0.42, 0.44, 0.52, 1.0];
pub const COLUMNS: i32 = 20;
pub const STEP: f32 = 1.2;
pub const LIFT: f32 = 3.0;
pub const WAVE: f32 = 0.5;
pub const PROBES: usize = 12;
pub const RING: f32 = 7.0;
pub const RATE: f32 = 6.0;
pub const BLOCK: f32 = 2.0;

pub const CAMERA_TARGET: Vec3 = Vec3::new(0.0, 2.0, 0.0);
pub const CAMERA_BACK: f32 = 32.0;
pub const CAMERA_YAW: f32 = 0.7;
pub const CAMERA_PITCH: f32 = 0.5;
