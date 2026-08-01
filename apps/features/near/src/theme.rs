use nalgebra_glm::Vec3;

pub const TITLE: &str = "Near";
pub const COLUMNS: i32 = 12;
pub const SPACING: f32 = 1.6;
pub const CROWD_TINT: [f32; 4] = [0.35, 0.55, 0.85, 1.0];
pub const SEEKER_TINT: [f32; 4] = [0.95, 0.72, 0.25, 1.0];
pub const FLOOR_TINT: [f32; 4] = [0.14, 0.14, 0.17, 1.0];
pub const BULK: f32 = 0.6;
pub const REACH: f32 = 6.0;
pub const WALK: f32 = 9.0;
pub const PUSH: f32 = 4.0;
pub const CELL: f32 = 4.0;
pub const ROOM: f32 = 2.2;
pub const CELL_CAPACITY: u32 = 32;
pub const SEEKER_TAG: u32 = 1;

pub const CAMERA_TARGET: Vec3 = Vec3::new(0.0, 0.0, 0.0);
pub const CAMERA_BACK: f32 = 34.0;
pub const CAMERA_YAW: f32 = 0.6;
pub const CAMERA_PITCH: f32 = 0.9;
