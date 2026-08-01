use nalgebra_glm::Vec3;

pub const TITLE: &str = "Platformer";
pub const REACH: f32 = 14.0;
pub const FLOOR_TINT: [f32; 4] = [0.15, 0.15, 0.18, 1.0];
pub const LEDGE_TINT: [f32; 4] = [0.30, 0.32, 0.38, 1.0];
pub const WALKER_TINT: [f32; 4] = [0.92, 0.70, 0.36, 1.0];
pub const COYOTE: f32 = 0.18;
pub const BUFFER: f32 = 0.18;
pub const GRAVITY: f32 = 22.0;
pub const JUMP: f32 = 9.0;
pub const RUN: f32 = 5.0;
pub const LEDGE: f32 = 4.0;
pub const LEDGE_TOP: f32 = 2.0;

pub const CAMERA_TARGET: Vec3 = Vec3::new(0.0, 2.0, 0.0);
pub const CAMERA_BACK: f32 = 20.0;
pub const CAMERA_YAW: f32 = 0.0;
pub const CAMERA_PITCH: f32 = 0.2;
