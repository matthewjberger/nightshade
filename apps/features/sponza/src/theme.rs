use nightshade::prelude::Vec3;

pub const TITLE: &str = "Sponza";

pub const PANEL_WIDTH: f32 = 280.0;
pub const PANEL_PADDING: f32 = 12.0;
pub const PANEL_SPACING: f32 = 8.0;

pub const MODEL: &str = "Sponza";

pub const CAMERA_TARGET: Vec3 = Vec3::new(2.0, 3.0, -0.8);
pub const CAMERA_DISTANCE: f32 = 10.1;
pub const CAMERA_YAW: f32 = -1.6;
pub const CAMERA_PITCH: f32 = -0.16;

pub const WALK_START: Vec3 = Vec3::new(1.0, 1.3, 3.4);
pub const EYE_HEIGHT: f32 = 0.6;
pub const WALK_SPEED: f32 = 3.5;
pub const EYE_FIELD_OF_VIEW: f32 = 70.0;

pub const HALF_TURN: f32 = std::f32::consts::PI;

pub const SUN_YAW: f32 = 1.1;
pub const SUN_PITCH: f32 = -1.15;
pub const SUN_COLOR: Vec3 = Vec3::new(1.0, 0.94, 0.86);
pub const SUN_INTENSITY: f32 = 30.0;

pub const LANTERNS: [Vec3; 3] = [
    Vec3::new(-5.0, 1.6, -2.6),
    Vec3::new(1.0, 1.6, 1.6),
    Vec3::new(6.0, 1.6, -2.6),
];
pub const LANTERN_COLOR: Vec3 = Vec3::new(1.0, 0.68, 0.38);
pub const LANTERN_INTENSITY: f32 = 40.0;
pub const LANTERN_RANGE: f32 = 12.0;

pub const FOG_COLOR: [f32; 3] = [0.72, 0.76, 0.82];
pub const FOG_END: f32 = 60.0;
pub const FOG_DENSITY: f32 = 0.03;
pub const FOG_ANISOTROPY: f32 = 0.65;
pub const FOG_HEIGHT: f32 = -0.5;
pub const FOG_HEIGHT_FALLOFF: f32 = 0.25;

pub const EXPOSURE_LEAST_STOPS: f32 = -7.0;
pub const EXPOSURE_MOST_STOPS: f32 = 4.0;

pub const FOCUS_NEAREST: f32 = 0.3;
pub const FOCUS_FARTHEST: f32 = 40.0;
pub const FOCUS_RATE: f32 = 6.0;
pub const FOCUS_RANGE_SHARE: f32 = 0.4;
pub const FOCUS_RANGE_LEAST: f32 = 0.35;
pub const FOCUS_RANGE_MOST: f32 = 8.0;
pub const FOCUS_BLUR_RADIUS: f32 = 4.0;

pub const SHARPEN: f32 = 0.2;
