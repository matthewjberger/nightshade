use nightshade_ui::prelude::*;

pub const TITLE: &str = "Debug views";

pub const CAMERA_BACK: f32 = 2.4;
pub const CAMERA_YAW: f32 = 0.6;
pub const CAMERA_PITCH: f32 = 0.25;

pub const SUN_YAW: f32 = 0.9;
pub const SUN_PITCH: f32 = -0.7;

pub const GROUND: f32 = 4.0;

pub const SKY: &[u8] = include_bytes!("../../../../assets/sky/winter_river_1k.hdr");

pub const MODEL: &str = "DamagedHelmet";

pub const PANEL_RECT: Rect = Rect {
    min: Vec2::new(16.0, 16.0),
    max: Vec2::new(300.0, 640.0),
};
