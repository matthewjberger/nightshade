use nalgebra_glm::{IVec2, Vec3};

pub const TITLE: &str = "Cells";
pub const WALL_TINT: [f32; 4] = [0.36, 0.34, 0.40, 1.0];
pub const OPEN_TINT: [f32; 4] = [0.18, 0.22, 0.26, 1.0];
pub const WALKER_TINT: [f32; 4] = [0.92, 0.70, 0.36, 1.0];
pub const MARK_TINT: [f32; 4] = [0.42, 0.78, 0.92, 1.0];
pub const SIDE: i32 = 11;
pub const STEP: f32 = 1.4;
pub const COST_RISE: f32 = 0.25;
pub const START: IVec2 = IVec2::new(1, 0);

pub const CAMERA_TARGET: Vec3 = Vec3::new(0.0, 0.0, 0.0);
pub const CAMERA_BACK: f32 = 22.0;
pub const CAMERA_YAW: f32 = 0.3;
pub const CAMERA_PITCH: f32 = 1.0;
