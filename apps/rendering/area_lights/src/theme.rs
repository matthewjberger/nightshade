use nightshade::prelude::Vec3;

pub const TITLE: &str = "Area lights";

pub const CAMERA_TARGET: Vec3 = Vec3::new(0.0, 1.5, 0.0);
pub const CAMERA_DISTANCE: f32 = 30.0;
pub const CAMERA_YAW: f32 = 0.55;
pub const CAMERA_PITCH: f32 = 0.42;

pub const SURFACE_DEPTH: f32 = 0.06;
pub const MARKER_GLOW: f32 = 3.0;

pub const PANELS_ALONG: usize = 8;
pub const PANELS_ACROSS: usize = 5;
pub const PANEL_PITCH: f32 = 4.0;
pub const PANEL_HEIGHT: f32 = 6.0;
pub const PANEL_WIDTH: f32 = 2.2;
pub const PANEL_DEPTH: f32 = 1.2;
pub const PANEL_RANGE: f32 = 11.0;
pub const PANEL_INTENSITY: f32 = 5.0;

pub const SPHERES: usize = 7;
pub const SPHERE_PITCH: f32 = 3.2;
pub const SPHERE_RADIUS: f32 = 1.1;

pub const TINTS: [Vec3; 5] = [
    Vec3::new(1.0, 0.62, 0.40),
    Vec3::new(0.45, 0.72, 1.0),
    Vec3::new(0.95, 0.45, 0.85),
    Vec3::new(0.55, 1.0, 0.70),
    Vec3::new(1.0, 0.92, 0.55),
];
