use nalgebra_glm::Vec3;

pub const TITLE: &str = "Arc";
pub const REACH: f32 = 12.0;
pub const FLOOR_TINT: [f32; 4] = [0.15, 0.15, 0.18, 1.0];
pub const SWINGER_TINT: [f32; 4] = [0.92, 0.70, 0.36, 1.0];
pub const TARGET_TINT: [f32; 4] = [0.42, 0.78, 0.92, 1.0];
pub const TARGETS: usize = 24;
pub const RING: f32 = 6.0;
pub const SWING_REACH: f32 = 7.0;
pub const SHORTEST_REACH: f32 = 2.0;
pub const LONGEST_REACH: f32 = 12.0;
pub const REACH_NUDGE: f32 = 0.5;
pub const HALF_ANGLE: f32 = 0.5;
pub const NARROWEST_ANGLE: f32 = 0.1;
pub const WIDEST_ANGLE: f32 = std::f32::consts::PI;
pub const ANGLE_NUDGE: f32 = 0.1;
pub const RATE: f32 = 1.6;
pub const STEP: f32 = 5.0;
pub const LIFT: f32 = 1.6;

pub const CAMERA_TARGET: Vec3 = Vec3::new(0.0, 1.0, 0.0);
pub const CAMERA_BACK: f32 = 24.0;
pub const CAMERA_YAW: f32 = 0.6;
pub const CAMERA_PITCH: f32 = 0.6;
