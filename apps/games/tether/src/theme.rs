use nightshade::prelude::Vec4;

pub const fn rgba(red: f32, green: f32, blue: f32, alpha: f32) -> [f32; 4] {
    [red, green, blue, alpha]
}

pub const ROPE_COLOR: [f32; 4] = rgba(0.96, 0.84, 0.52, 1.0);
pub const ROPE_GLOW: [f32; 3] = [0.9, 0.7, 0.3];
pub const BEACON_COLOR: [f32; 4] = rgba(0.42, 0.92, 1.0, 1.0);
pub const BEACON_GLOW: [f32; 3] = [0.22, 1.35, 1.85];
pub const RUNNER_COLOR: [f32; 4] = rgba(0.12, 0.14, 0.2, 1.0);

pub const FOG_COLOR: [f32; 3] = [0.28, 0.20, 0.26];
pub const FOG_START: f32 = 60.0;
pub const FOG_END: f32 = 420.0;
pub const FOG_DENSITY: f32 = 0.0062;

pub const SUN_TINT: [f32; 3] = [1.0, 0.86, 0.68];
pub const SUN_INTENSITY: f32 = 4.2;
pub const SUN_SHADOW_DISTANCE: f32 = 160.0;

pub const BLOOM_INTENSITY: f32 = 0.07;
pub const BLOOM_THRESHOLD: f32 = 1.15;
pub const EXPOSURE_STOPS: f32 = -0.85;
pub const VIGNETTE_INTENSITY: f32 = 0.34;
pub const VIGNETTE_RADIUS: f32 = 0.8;
pub const SATURATION: f32 = 1.12;
pub const CONTRAST: f32 = 1.08;

pub const CLEAR: Vec4 = Vec4::new(0.0, 0.0, 0.0, 0.0);
pub const PANEL_BACKGROUND: Vec4 = Vec4::new(0.04, 0.04, 0.07, 0.62);
pub const MENU_BACKGROUND: Vec4 = Vec4::new(0.03, 0.03, 0.06, 0.93);
pub const PANEL_BORDER: Vec4 = Vec4::new(1.0, 1.0, 1.0, 0.1);
pub const TEXT_BRIGHT: Vec4 = Vec4::new(0.96, 0.97, 1.0, 1.0);
pub const TEXT_DIM: Vec4 = Vec4::new(0.66, 0.70, 0.80, 1.0);
pub const METER_TRACK: Vec4 = Vec4::new(1.0, 1.0, 1.0, 0.12);

pub const SPEED_BANDS: [(f32, Vec4); 4] = [
    (56.0, Vec4::new(1.0, 0.42, 0.86, 1.0)),
    (38.0, Vec4::new(1.0, 0.72, 0.30, 1.0)),
    (20.0, Vec4::new(0.52, 0.94, 0.66, 1.0)),
    (0.0, Vec4::new(0.70, 0.78, 0.90, 1.0)),
];

pub const RETICLE_IDLE: Vec4 = Vec4::new(1.0, 1.0, 1.0, 0.55);
pub const RETICLE_EDGE: Vec4 = Vec4::new(0.0, 0.0, 0.0, 0.6);
pub const RETICLE_READY: Vec4 = Vec4::new(0.42, 0.92, 1.0, 0.95);
pub const RETICLE_HELD: Vec4 = Vec4::new(0.96, 0.84, 0.52, 1.0);

pub const MARGIN: f32 = 22.0;
pub const PADDING: f32 = 16.0;
pub const PANEL_WIDTH: f32 = 300.0;
pub const PANEL_HEIGHT: f32 = 96.0;
pub const SPEED_SIZE: f32 = 40.0;
pub const SPEED_ROW_HEIGHT: f32 = 44.0;
pub const LABEL_SIZE: f32 = 15.0;
pub const LABEL_TOP: f32 = 68.0;
pub const METER_WIDTH: f32 = 256.0;
pub const METER_HEIGHT: f32 = 5.0;
pub const METER_TOP: f32 = 56.0;
pub const RETICLE_SIZE: f32 = 11.0;
pub const RETICLE_EDGE_WIDTH: f32 = 1.5;

pub const MENU_CORNER: f32 = 10.0;
pub const MENU_BORDER_WIDTH: f32 = 1.0;
pub const MENU_PADDING: f32 = 18.0;
pub const MENU_SPACING: f32 = 10.0;
pub const MENU_HEADING_SIZE: f32 = 34.0;
pub const MENU_ROW_SIZE: f32 = 15.0;
pub const TITLE_WIDTH: f32 = 360.0;
pub const TITLE_HEIGHT: f32 = 224.0;
pub const PAUSE_WIDTH: f32 = 320.0;
pub const PAUSE_HEIGHT: f32 = 268.0;
