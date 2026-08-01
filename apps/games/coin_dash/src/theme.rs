use nightshade::prelude::*;

pub const PANEL_BG: Vec4 = Vec4::new(0.05, 0.05, 0.08, 0.85);
pub const PANEL_BORDER: Vec4 = Vec4::new(1.0, 1.0, 1.0, 0.1);
pub const TEXT_PRIMARY: Vec4 = Vec4::new(1.0, 1.0, 1.0, 1.0);

pub const GRAY: [f32; 4] = [0.5, 0.5, 0.5, 1.0];
pub const STEEL: [f32; 4] = [0.45, 0.5, 0.6, 1.0];
pub const GREEN: [f32; 4] = [0.2, 0.7, 0.3, 1.0];
pub const GOLD: [f32; 4] = [1.0, 0.85, 0.2, 1.0];

pub const fn rgb(red: f32, green: f32, blue: f32) -> [f32; 4] {
    [red, green, blue, 1.0]
}

pub const HUD_WIDTH: f32 = 340.0;
pub const HUD_HEIGHT: f32 = 56.0;
pub const TITLE_WIDTH: f32 = 540.0;
pub const TITLE_HEIGHT: f32 = 240.0;
pub const PAUSE_WIDTH: f32 = 300.0;
pub const PAUSE_HEIGHT: f32 = 210.0;
pub const OVER_WIDTH: f32 = 340.0;
pub const OVER_HEIGHT: f32 = 210.0;

pub const TITLE_SIZE: f32 = 36.0;
pub const HEADING_SIZE: f32 = 32.0;
pub const HUD_SIZE: f32 = 22.0;
pub const LABEL_SIZE: f32 = 18.0;
pub const ROW_HEIGHT: f32 = 24.0;
