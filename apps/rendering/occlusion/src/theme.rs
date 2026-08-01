use nightshade::prelude::Vec3;

pub const TITLE: &str = "Occlusion culling";

pub const CAMERA_TARGET: Vec3 = Vec3::new(0.0, 6.0, -6.0);
pub const CAMERA_DISTANCE: f32 = 48.0;
pub const CAMERA_YAW: f32 = 0.0;
pub const CAMERA_PITCH: f32 = 0.14;

pub const HIDDEN: usize = 4000;
pub const ACROSS: usize = 60;
pub const WALL: f32 = 34.0;
