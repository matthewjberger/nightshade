use nalgebra_glm::Vec3;

pub const TITLE: &str = "Weather";
pub const REACH: f32 = 14.0;
pub const FLOOR_TINT: [f32; 4] = [0.15, 0.15, 0.18, 1.0];
pub const CLOUD_TINT: [f32; 4] = [0.68, 0.70, 0.76, 1.0];
pub const CLOUDS: usize = 12;
pub const SPREAD: f32 = 9.0;
pub const CEILING: f32 = 7.0;
pub const SHORTEST: f32 = 3.0;
pub const LONGEST: f32 = 6.0;
pub const CHANCE_OF_RAIN: f32 = 0.5;
pub const DRY_RATE: f32 = 0.15;

pub const CAMERA_TARGET: Vec3 = Vec3::new(0.0, 3.0, 0.0);
pub const CAMERA_BACK: f32 = 26.0;
pub const CAMERA_YAW: f32 = 0.4;
pub const CAMERA_PITCH: f32 = 0.35;

pub const CLOUD_FLOOR: f32 = 5.0;
pub const DESCENT: f32 = 2.6;
pub const GATHER: f32 = 2.5;
pub const DRIFT: f32 = 0.12;
pub const SKY_CLEAR: [f32; 3] = [0.42, 0.62, 0.86];
pub const SKY_STORM: [f32; 3] = [0.10, 0.11, 0.14];
pub const FOG_START: f32 = 8.0;
pub const FOG_END: f32 = 60.0;
pub const FOG_CLOSE: f32 = 32.0;

pub const HELD: f32 = 999.0;
