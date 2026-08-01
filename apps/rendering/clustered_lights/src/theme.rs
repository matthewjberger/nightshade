use nightshade::prelude::Vec3;

pub const TITLE: &str = "Clustered lights";

pub const CAMERA_TARGET: Vec3 = Vec3::new(0.0, 3.0, 0.0);
pub const CAMERA_DISTANCE: f32 = 72.0;
pub const CAMERA_YAW: f32 = 0.8;
pub const CAMERA_PITCH: f32 = 0.45;

pub const LIGHTS: usize = 512;
pub const BLOCKS: usize = 220;
pub const SPREAD: f32 = 46.0;

pub const SHADOW_LIGHTS: usize = 12;
pub const SHADOW_HEIGHT: f32 = 6.5;
pub const SHADOW_RING: f32 = 14.0;
pub const SHADOW_RANGE: f32 = 22.0;

pub const TINTS: [Vec3; 6] = [
    Vec3::new(1.0, 0.35, 0.22),
    Vec3::new(0.22, 0.70, 1.0),
    Vec3::new(0.90, 0.28, 1.0),
    Vec3::new(0.32, 1.0, 0.58),
    Vec3::new(1.0, 0.86, 0.30),
    Vec3::new(1.0, 0.22, 0.44),
];
