use nightshade::prelude::Vec3;

pub const TITLE: &str = "Instancing";

pub const MESH: &str = "Cube";

pub const SIDE: usize = 100;
pub const COPIES: usize = SIDE * SIDE * SIDE;
pub const PITCH: f32 = 2.0;
pub const SCALE: f32 = 0.42;

pub const CAMERA_TARGET: Vec3 = Vec3::new(0.0, 0.0, 0.0);
pub const CAMERA_DISTANCE: f32 = 330.0;
pub const CAMERA_YAW: f32 = 0.7;
pub const CAMERA_PITCH: f32 = 0.5;

pub const SUN_YAW: f32 = 0.9;
pub const SUN_PITCH: f32 = -0.8;
