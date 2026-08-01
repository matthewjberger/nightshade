use nightshade_ui::prelude::*;

pub const MESH_LAYOUT: [(&str, Vec3, Vec4); 5] = [
    (
        "Cube",
        Vec3::new(-8.0, 0.0, 0.0),
        Vec4::new(0.8, 0.2, 0.2, 1.0),
    ),
    (
        "Sphere",
        Vec3::new(-4.0, 0.0, 0.0),
        Vec4::new(0.2, 0.8, 0.2, 1.0),
    ),
    (
        "Cylinder",
        Vec3::new(0.0, 0.0, 0.0),
        Vec4::new(0.2, 0.2, 0.8, 1.0),
    ),
    (
        "Torus",
        Vec3::new(4.0, 0.0, 0.0),
        Vec4::new(0.2, 0.8, 0.8, 1.0),
    ),
    (
        "Cone",
        Vec3::new(8.0, 0.0, 0.0),
        Vec4::new(0.8, 0.2, 0.8, 1.0),
    ),
];

pub const PANEL_RECT: Rect = Rect {
    min: Vec2::new(16.0, 16.0),
    max: Vec2::new(360.0, 320.0),
};

pub const DEBUG_VOLUME_COLOR: Vec4 = Vec4::new(0.0, 1.0, 1.0, 1.0);

pub const DEBUG_VOLUME_SEGMENTS: u32 = 16;

pub const PREVIEW_MARKER_SIZE: f32 = 0.6;

pub const CAMERA_TARGET: Vec3 = Vec3::new(0.0, 0.0, 0.0);
pub const CAMERA_DISTANCE: f32 = 12.0;
pub const CAMERA_YAW: f32 = 0.0;
pub const CAMERA_PITCH: f32 = 0.4;
