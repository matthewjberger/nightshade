use nightshade::prelude::Vec3;

pub const TITLE: &str = "Meshlet detail levels";

pub const CAMERA_TARGET: Vec3 = Vec3::new(0.0, 4.0, -80.0);
pub const CAMERA_DISTANCE: f32 = 100.0;
pub const CAMERA_YAW: f32 = 0.10;
pub const CAMERA_PITCH: f32 = 0.13;

pub const SKY: &[u8] = include_bytes!("../../../../assets/sky/winter_river_1k.hdr");

pub const MODEL: &str = "AntiqueCamera";

pub const COPIES: usize = 3000;

pub const ACROSS: usize = 30;

pub const SPACING_ACROSS: f32 = 3.6;
pub const SPACING_AWAY: f32 = 4.4;
