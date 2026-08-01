use nightshade::prelude::Vec3;

nightshade_config::config! {
    pub enum Settings {
        "Motion" {
            SpinSpeed: number(0.0, 4.0, 1.0),
            Bob: toggle(true),
            BobHeight: number(0.0, 1.5, 0.4),
        }
        "Scene" {
            Cubes: whole(1, 8, 5),
            Spacing: number(1.5, 4.0, 2.4),
            Palette: choice(0, ["Ember", "Meadow", "Ice"]),
            Tint: color([1.0, 1.0, 1.0, 1.0]),
        }
        "Panel" {
            Title: text("Settings"),
        }
    }
}

pub const PALETTE_COLORS: [[f32; 4]; 3] = [
    [0.92, 0.44, 0.18, 1.0],
    [0.36, 0.72, 0.38, 1.0],
    [0.42, 0.68, 0.94, 1.0],
];

pub const PALETTE_SHADES: [f32; 4] = [1.0, 0.84, 0.68, 0.52];

pub const CUBE_SCALE: Vec3 = Vec3::new(1.0, 1.0, 1.0);

pub const CUBE_ORIGIN: Vec3 = Vec3::new(0.0, 0.5, 0.0);

pub const CAMERA_TARGET: Vec3 = Vec3::new(0.0, 0.5, 0.0);
pub const CAMERA_DISTANCE: f32 = 14.0;
pub const CAMERA_YAW: f32 = 0.6;
pub const CAMERA_PITCH: f32 = 0.42;

pub const PANEL_WIDTH: f32 = 320.0;
pub const PANEL_PADDING: f32 = 12.0;
pub const PANEL_SPACING: f32 = 8.0;

pub const BOB_RATE: f32 = 1.6;
