use nalgebra_glm::Vec3;

pub const TITLE: &str = "Inventory";
pub const REACH: f32 = 12.0;
pub const FLOOR_TINT: [f32; 4] = [0.15, 0.15, 0.18, 1.0];
pub const SLOT_TINT: [f32; 4] = [0.22, 0.24, 0.30, 1.0];
pub const HELD_TINT: [f32; 4] = [0.92, 0.70, 0.36, 1.0];
pub const SLOTS: usize = 6;
pub const SPACING: f32 = 1.8;
pub const STACK: u32 = 8;
pub const KINDS: [u32; 3] = [1, 2, 3];
pub const HANDFUL: u32 = 1;

pub const CAMERA_TARGET: Vec3 = Vec3::new(0.0, 1.0, 0.0);
pub const CAMERA_BACK: f32 = 18.0;
pub const CAMERA_YAW: f32 = 0.0;
pub const CAMERA_PITCH: f32 = 0.5;
