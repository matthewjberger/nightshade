use nalgebra_glm::Vec3;

pub const TITLE: &str = "Flicker";
pub const REACH: f32 = 16.0;
pub const FLOOR_TINT: [f32; 4] = [0.12, 0.11, 0.12, 1.0];
pub const POST_TINT: [f32; 4] = [0.30, 0.28, 0.26, 1.0];
pub const LAMPS: usize = 5;
pub const SPACING: f32 = 5.0;
pub const LAMP_HEIGHT: f32 = 3.6;
pub const LAMP_COLOR: Vec3 = Vec3::new(1.0, 0.72, 0.45);
pub const LAMP_RANGE: f32 = 14.0;
pub const BRIGHTEST: f32 = 90.0;
pub const DIMMEST: f32 = 30.0;
pub const DEPTH: f32 = 0.35;
pub const DROPOUT_CHANCE: f32 = 0.04;
pub const DROPOUT_SCALE: f32 = 0.12;
pub const RESPONSE: f32 = 18.0;
pub const DARK_BELOW: f32 = 0.45;
pub const STEP: f32 = 0.05;

pub const CAMERA_TARGET: Vec3 = Vec3::new(0.0, 2.0, 0.0);
pub const CAMERA_BACK: f32 = 22.0;
pub const CAMERA_YAW: f32 = 0.4;
pub const CAMERA_PITCH: f32 = 0.22;
