use nalgebra_glm::Vec3;

pub const TITLE: &str = "Settings panel";
pub const REACH: f32 = 12.0;
pub const FLOOR_TINT: [f32; 4] = [0.15, 0.15, 0.18, 1.0];
pub const BODY_TINT: [f32; 4] = [0.74, 0.70, 0.62, 1.0];
pub const PANEL_WIDTH: f32 = 320.0;
pub const PANEL_PADDING: f32 = 12.0;
pub const PANEL_SPACING: f32 = 8.0;
pub const TINTS: [[f32; 4]; 3] = [
    [0.92, 0.44, 0.18, 1.0],
    [0.36, 0.72, 0.38, 1.0],
    [0.42, 0.68, 0.94, 1.0],
];

pub const CAMERA_TARGET: Vec3 = Vec3::new(0.0, 1.2, 0.0);
pub const CAMERA_BACK: f32 = 16.0;
pub const CAMERA_YAW: f32 = 0.3;
pub const CAMERA_PITCH: f32 = 0.35;

pub const PANEL_TITLE: &str = "Settings";
