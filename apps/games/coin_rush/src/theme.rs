use nightshade::prelude::*;

pub const PANEL_BG: Vec4 = Vec4::new(0.05, 0.05, 0.08, 0.85);
pub const PANEL_BORDER: Vec4 = Vec4::new(1.0, 1.0, 1.0, 0.1);
pub const TEXT_PRIMARY: Vec4 = Vec4::new(1.0, 1.0, 1.0, 1.0);

pub const PLAYER_COLOR: [f32; 4] = [0.1, 0.65, 0.65, 1.0];
pub const COIN_COLOR: [f32; 4] = [1.0, 0.85, 0.2, 1.0];

pub const TITLE_WIDTH: f32 = 460.0;
pub const TITLE_HEIGHT: f32 = 220.0;
pub const MENU_WIDTH: f32 = 300.0;
pub const MENU_HEIGHT: f32 = 210.0;

pub const TITLE_SIZE: f32 = 36.0;
pub const HEADING_SIZE: f32 = 32.0;
pub const LABEL_SIZE: f32 = 18.0;

pub const PLAYER_SPEED: f32 = 10.0;

pub const ARENA: f32 = 14.0;
pub const ROUND_SECONDS: f32 = 30.0;
pub const FIRST_GOAL: u32 = 4;
pub const GOAL_STEP: u32 = 2;
pub const PLAYER_START: Vec3 = Vec3::new(0.0, 0.5, 0.0);
pub const COIN_HEIGHT: f32 = 0.7;
pub const COIN_KIND: u32 = 0;
pub const COIN_GRASP: f32 = 1.0;
pub const COIN_BOB: f32 = 0.25;
pub const COIN_PHASE: f32 = 0.7;
pub const COIN_SPIN: f32 = 3.0;
pub const COIN_RATE: f32 = 1.5;
