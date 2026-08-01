use nightshade_ui::prelude::*;

pub const TITLE: &str = "Text style";

pub const PANEL_RECT: Rect = Rect {
    min: Vec2::new(16.0, 16.0),
    max: Vec2::new(340.0, 520.0),
};

pub const CAMERA_TARGET: Vec3 = Vec3::new(0.0, 1.0, 0.0);
pub const CAMERA_DISTANCE: f32 = 9.0;
pub const CAMERA_YAW: f32 = 0.0;
pub const CAMERA_PITCH: f32 = 0.2;

pub const TEXT_COLOR: Vec4 = Vec4::new(1.0, 0.95, 0.85, 1.0);

pub const WIDEST_OUTLINE: f32 = 8.0;
pub const SOFTEST_EDGE: f32 = 3.0;
pub const LARGEST_SCALE: f32 = 4.0;
