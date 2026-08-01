use nightshade::prelude::Vec3;

pub const TITLE: &str = "Volumetric fog";

pub const CAMERA_TARGET: Vec3 = Vec3::new(0.0, 3.0, 0.0);
pub const CAMERA_DISTANCE: f32 = 20.0;
pub const CAMERA_YAW: f32 = std::f32::consts::PI;
pub const CAMERA_PITCH: f32 = 0.10;

pub const SUN_YAW: f32 = 0.070;
pub const SUN_PITCH: f32 = -0.140;
pub const SUN_COLOR: Vec3 = Vec3::new(1.0, 0.93, 0.82);
pub const SUN_INTENSITY: f32 = 9.0;

pub const PILLARS: i32 = 7;
pub const PILLAR_SPACING: f32 = 10.0;
pub const PILLAR_BEYOND: f32 = 6.0;
pub const PILLAR_THICKNESS: f32 = 3.5;
pub const PILLAR_HEIGHT: f32 = 10.0;

pub const GROUND: f32 = 60.0;

pub const LANTERN: Vec3 = Vec3::new(-3.2, 1.6, -3.2);
pub const LANTERN_COLOR: Vec3 = Vec3::new(1.0, 0.70, 0.40);
pub const LANTERN_INTENSITY: f32 = 40.0;
pub const LANTERN_RANGE: f32 = 14.0;

pub const MODEL: &str = "DamagedHelmet";
pub const MODEL_REACH: f32 = 2.5;

pub const FOG_COLOR: [f32; 3] = [0.15, 0.16, 0.20];
pub const FOG_END: f32 = 60.0;
pub const FOG_DENSITY: f32 = 0.035;
pub const FOG_ANISOTROPY: f32 = 0.60;
pub const FOG_HEIGHT: f32 = 0.0;
pub const FOG_HEIGHT_FALLOFF: f32 = 0.06;
