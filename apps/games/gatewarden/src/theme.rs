use crate::data::cards::Element;
use crate::data::climb::{COLUMNS, LANES};
use nightshade::prelude::*;

pub const PANEL_BG: Vec4 = Vec4::new(0.075, 0.085, 0.115, 0.95);
pub const PANEL_BG_BOTTOM: Vec4 = Vec4::new(0.032, 0.042, 0.06, 0.97);
pub const PANEL_BORDER: Vec4 = Vec4::new(0.78, 0.63, 0.35, 0.42);
pub const FRAME_TINT: Vec4 = Vec4::new(0.86, 0.72, 0.43, 0.92);
pub const FRAME_TINT_SOFT: Vec4 = Vec4::new(0.82, 0.7, 0.45, 0.6);
pub const BUTTON_FRAME_TINT: Vec4 = Vec4::new(0.97, 0.9, 0.7, 0.5);
pub const FRAME_CORNER: f32 = 26.0;
pub const PANEL_FRAME_CORNER: f32 = 20.0;
pub const BUTTON_FRAME_CORNER: f32 = 14.0;
pub const TEXT_PRIMARY: Vec4 = Vec4::new(0.94, 0.92, 0.86, 1.0);
pub const TEXT_DIM: Vec4 = Vec4::new(0.66, 0.65, 0.62, 1.0);
pub const TEXT_GOLD: Vec4 = Vec4::new(0.96, 0.8, 0.4, 1.0);
pub const TEXT_OUTLINE: Vec4 = Vec4::new(0.02, 0.025, 0.04, 1.0);
pub const CARD_TEXT_COLOR: Vec4 = Vec4::new(0.16, 0.12, 0.1, 1.0);
pub const INTENT_CALM: Vec4 = Vec4::new(0.86, 0.82, 0.7, 0.9);
pub const INTENT_THREAT: Vec4 = Vec4::new(1.0, 0.72, 0.34, 1.0);
pub const INTENT_LETHAL: Vec4 = Vec4::new(1.0, 0.36, 0.32, 1.0);
pub const PREVIEW_HURT: Vec4 = Vec4::new(1.0, 0.85, 0.4, 1.0);
pub const PREVIEW_KILL: Vec4 = Vec4::new(1.0, 0.42, 0.36, 1.0);
pub const TEXT_FORGED: Vec4 = Vec4::new(0.42, 0.24, 0.06, 1.0);

pub const TABLE_BG: Vec4 = Vec4::new(0.055, 0.1, 0.085, 0.55);
pub const TABLE_BG_BOTTOM: Vec4 = Vec4::new(0.022, 0.048, 0.04, 0.74);
pub const TABLE_BORDER: Vec4 = Vec4::new(0.78, 0.63, 0.35, 0.26);

pub const CARD_FACE: Vec4 = Vec4::new(0.94, 0.9, 0.79, 1.0);
pub const CARD_FACE_BOTTOM: Vec4 = Vec4::new(0.83, 0.77, 0.63, 1.0);
pub const CARD_FACE_LIT: Vec4 = Vec4::new(1.0, 0.97, 0.88, 1.0);
pub const CARD_FACE_ENEMY: Vec4 = Vec4::new(0.9, 0.84, 0.78, 1.0);
pub const CARD_SHADOW: Vec4 = Vec4::new(0.0, 0.0, 0.0, 0.55);
pub const CARD_WASH_ALPHA: f32 = 0.2;
pub const CARD_DIM: Vec4 = Vec4::new(0.03, 0.03, 0.05, 0.55);
pub const CARD_FRAME_TINT: Vec4 = Vec4::new(0.4, 0.31, 0.17, 0.8);
pub const CARD_FRAME_INSET: f32 = 3.0;
pub const CARD_FRAME_CORNER: f32 = 13.0;
pub const MINION_FRAME_CORNER: f32 = 12.0;
pub const GEM_RING: Vec4 = Vec4::new(0.06, 0.05, 0.09, 0.9);
pub const RULE_HEIGHT: f32 = 2.0;
pub const RULE_COLOR: Vec4 = Vec4::new(0.45, 0.35, 0.19, 0.75);
pub const KEYWORD_MARK_SIZE: f32 = 14.0;
pub const KEYWORD_MARK_GAP: f32 = 3.0;
pub const KEYWORD_MARK_FILL: Vec4 = Vec4::new(0.1, 0.095, 0.13, 0.85);
pub const MINION_PLATE_HEIGHT: f32 = 13.0;
pub const MINION_PLATE_SPREAD: f32 = 10.0;
pub const MINION_PLATE_COLOR: Vec4 = Vec4::new(0.02, 0.03, 0.04, 0.42);
pub const HAND_ARC: f32 = 9.0;
pub const ARROW_SEGMENTS: usize = 18;
pub const ARROW_LINE_MIN: f32 = 3.0;
pub const ARROW_LINE_MAX: f32 = 7.0;
pub const ARROW_HEAD_LENGTH: f32 = 22.0;
pub const ARROW_HEAD_WIDTH: f32 = 22.0;
pub const ARROW_HEAD_LINES: usize = 12;
pub const ARROW_BOW: f32 = 0.28;
pub const ARROW_BOW_MAX: f32 = 130.0;
pub const ARROW_COLOR: Vec4 = Vec4::new(1.0, 0.86, 0.36, 0.95);
pub const ARROW_HEAD_COLOR: Vec4 = Vec4::new(1.0, 0.94, 0.62, 1.0);
pub const ARROW_DEPTH: f32 = 420.0;
pub const CARD_DEPTH_BASE: f32 = 20.0;
pub const CARD_DEPTH_STRIDE: f32 = 8.0;
pub const CARD_RANK_PLAYER_ROW: f32 = 8.0;
pub const CARD_RANK_HAND: f32 = 16.0;
pub const CARD_RANK_LIFTED: f32 = 32.0;
pub const CARD_RANK_LUNGE: f32 = 40.0;
pub const HAND_FAN: f32 = 0.05;
pub const MANA_BLUE: Vec4 = Vec4::new(0.2, 0.45, 0.9, 1.0);
pub const MANA_BLUE_DEEP: Vec4 = Vec4::new(0.1, 0.24, 0.62, 1.0);
pub const MANA_BAR_EDGE: Vec4 = Vec4::new(0.45, 0.6, 0.95, 0.65);
pub const MANA_SPENT: Vec4 = Vec4::new(0.1, 0.12, 0.22, 1.0);
pub const BAR_TRACK: Vec4 = Vec4::new(0.04, 0.045, 0.07, 1.0);
pub const CARD_BACK: Vec4 = Vec4::new(0.16, 0.2, 0.42, 1.0);
pub const CARD_BACK_BORDER: Vec4 = Vec4::new(0.7, 0.75, 1.0, 0.6);
pub const FLOAT_DAMAGE: Vec4 = Vec4::new(1.0, 0.35, 0.3, 1.0);
pub const FLOAT_HEAL: Vec4 = Vec4::new(0.4, 1.0, 0.55, 1.0);
pub const FLOAT_SHIELD: Vec4 = Vec4::new(0.55, 0.9, 1.0, 1.0);
pub const FLOAT_FALLS: Vec4 = Vec4::new(0.82, 0.76, 0.7, 1.0);
pub const TEXT_WARN: Vec4 = Vec4::new(1.0, 0.55, 0.45, 1.0);
pub const ATTACK_GOLD: Vec4 = Vec4::new(0.86, 0.65, 0.22, 1.0);
pub const HEALTH_RED: Vec4 = Vec4::new(0.74, 0.2, 0.18, 1.0);

pub const BORDER_SELECTED: Vec4 = Vec4::new(1.0, 0.85, 0.3, 1.0);
pub const BORDER_TARGET: Vec4 = Vec4::new(1.0, 0.3, 0.9, 1.0);
pub const BORDER_TAUNT: Vec4 = Vec4::new(0.55, 0.55, 0.6, 1.0);
pub const BORDER_SHIELD: Vec4 = Vec4::new(0.5, 0.9, 1.0, 1.0);
pub const BORDER_CURSOR: Vec4 = Vec4::new(1.0, 1.0, 1.0, 1.0);
pub const BORDER_READY: Vec4 = Vec4::new(0.35, 0.9, 0.45, 1.0);
pub const CARD_BORDER_WIDTH: f32 = 2.0;
pub const CARD_BORDER_WIDTH_STRONG: f32 = 4.0;

pub const NEUTRAL: Vec4 = Vec4::new(0.44, 0.42, 0.4, 1.0);
pub const EMBER: Vec4 = Vec4::new(0.79, 0.3, 0.16, 1.0);
pub const TIDE: Vec4 = Vec4::new(0.16, 0.44, 0.74, 1.0);
pub const GROVE: Vec4 = Vec4::new(0.2, 0.54, 0.32, 1.0);
pub const STORM: Vec4 = Vec4::new(0.5, 0.32, 0.74, 1.0);

pub fn element_color(element: Element) -> Vec4 {
    match element {
        Element::Neutral => NEUTRAL,
        Element::Ember => EMBER,
        Element::Tide => TIDE,
        Element::Grove => GROVE,
        Element::Storm => STORM,
    }
}

pub fn portrait_fill(color: Vec4) -> Vec4 {
    Vec4::new(color.x * 0.62, color.y * 0.62, color.z * 0.62, 1.0)
}

pub const PORTRAIT_GLOW: Vec4 = Vec4::new(1.0, 0.92, 0.72, 1.0);
pub const PLAYER_SIGIL: char = '\u{e2aa}';
pub const CLIMB_SIGIL: char = '\u{e231}';
pub const SUMMIT_SIGIL: char = '\u{e232}';
pub const SHARD_SIGIL: char = '\u{e242}';
pub const SHIELD_SIGIL: char = '\u{e158}';
pub const RUSH_SIGIL: char = '\u{e1b0}';
pub const VENOM_SIGIL: char = '\u{e20c}';
pub const DRAIN_SIGIL: char = '\u{e0b4}';
pub const WHOLE_SIGIL: char = '\u{e412}';
pub const GATES_SIGIL: char = '\u{e3e0}';

pub const BUTTON_PLAY: Vec4 = Vec4::new(0.15, 0.42, 0.28, 1.0);
pub const BUTTON_END: Vec4 = Vec4::new(0.58, 0.36, 0.13, 1.0);
pub const BUTTON_POWER: Vec4 = Vec4::new(0.3, 0.24, 0.5, 1.0);
pub const BUTTON_NEUTRAL: Vec4 = Vec4::new(0.19, 0.2, 0.28, 1.0);
pub const BUTTON_BUY: Vec4 = Vec4::new(0.56, 0.42, 0.16, 1.0);
pub const BUTTON_BORDER: Vec4 = Vec4::new(1.0, 0.9, 0.7, 0.2);

pub const HERO_BG: Vec4 = Vec4::new(0.1, 0.105, 0.145, 0.96);
pub const HERO_BG_BOTTOM: Vec4 = Vec4::new(0.045, 0.05, 0.075, 0.98);
pub const RELIC_BG: Vec4 = Vec4::new(0.24, 0.16, 0.32, 0.96);
pub const RELIC_BG_BOTTOM: Vec4 = Vec4::new(0.13, 0.08, 0.19, 0.98);
pub const RELIC_BORDER: Vec4 = Vec4::new(0.86, 0.7, 0.4, 0.45);
pub const OFFER_BG: Vec4 = Vec4::new(0.115, 0.12, 0.175, 0.96);
pub const OFFER_BG_BOTTOM: Vec4 = Vec4::new(0.06, 0.065, 0.1, 0.98);
pub const BANNER_BG: Vec4 = Vec4::new(0.1, 0.075, 0.15, 0.94);
pub const BANNER_BG_BOTTOM: Vec4 = Vec4::new(0.04, 0.03, 0.07, 0.97);

pub const CARD_WIDTH: f32 = 96.0;
pub const CARD_HEIGHT: f32 = 192.0;
pub const CARD_GAP: f32 = 10.0;
pub const HAND_SIZE: usize = 8;
pub const HAND_WIDTH: f32 = CARD_WIDTH * HAND_SIZE as f32 + CARD_GAP * (HAND_SIZE as f32 - 1.0);
pub const MINION_WIDTH: f32 = 88.0;
pub const MINION_HEIGHT: f32 = 116.0;
pub const MINION_GAP: f32 = 12.0;
pub const ARENA_HEIGHT: f32 = 530.0;
pub const ENEMY_ROW_Y: f32 = 16.0;
pub const PLAYER_ROW_Y: f32 = ENEMY_ROW_Y + MINION_HEIGHT + 38.0;
pub const HAND_Y: f32 = ARENA_HEIGHT - CARD_HEIGHT - 4.0;
pub const HAND_PANEL_TOP: f32 = HAND_Y - 14.0;
pub const HAND_PANEL_HEIGHT: f32 = ARENA_HEIGHT - HAND_PANEL_TOP;
pub const CARD_MANA_INSET: f32 = 14.0;
pub const CARD_ART_Y: f32 = 61.0;
pub const CARD_ART_SIZE: f32 = 42.0;
pub const CARD_MARKS_Y: f32 = 85.0;
pub const CARD_NAME_Y: f32 = 102.0;
pub const CARD_RULE_Y: f32 = 122.0;
pub const CARD_TEXT_Y: f32 = 128.0;
pub const CARD_TEXT_HEIGHT: f32 = 24.0;
pub const MINION_ART_Y: f32 = 49.0;
pub const MINION_ART_SIZE: f32 = 32.0;
pub const MINION_MARKS_Y: f32 = 67.0;
pub const HOVER_RAISE: f32 = 16.0;
pub const SELECT_RAISE: f32 = 34.0;
pub const HAND_BOTTOM_MARGIN: f32 = 34.0;
pub const DEAL_SPEED: f32 = 11.0;
pub const DEAL_OFFSCREEN: f32 = 320.0;
pub const ENEMY_SPAWN_Y: f32 = -180.0;
pub const LUNGE_SECONDS: f32 = 0.42;
pub const LUNGE_REACH: f32 = 0.7;

pub const BUTTON_HEIGHT: f32 = 42.0;
pub const HERO_PANEL_WIDTH: f32 = 270.0;
pub const HERO_PANEL_HEIGHT: f32 = 206.0;
pub const CONTROLS_HEIGHT: f32 = 206.0;
pub const ENEMY_PANEL_WIDTH: f32 = 400.0;
pub const ENEMY_PANEL_HEIGHT: f32 = 220.0;
pub const PANEL_PADDING: f32 = 12.0;
pub const PORTRAIT_GAP: f32 = 12.0;
pub const ENEMY_PORTRAIT: f32 = 96.0;
pub const PLAYER_PORTRAIT: f32 = 72.0;
pub const SCENE_PORTRAIT: f32 = 88.0;
pub const PORTRAIT_RING: Vec4 = Vec4::new(0.92, 0.78, 0.46, 0.7);
pub const PORTRAIT_RING_WIDTH: f32 = 3.0;
pub const PREVIEW_HEIGHT: f32 = 168.0;
pub const LOG_HEIGHT: f32 = 216.0;
pub const LOG_LINES: usize = 24;
pub const LOG_VIEW_HEIGHT: f32 = 164.0;
pub const LOG_LINE_SIZE: f32 = 13.0;
pub const HAND_BACK_WIDTH: f32 = 26.0;
pub const HAND_BACK_HEIGHT: f32 = 36.0;
pub const MANA_BAR_HEIGHT: f32 = 22.0;
pub const MANA_BAR_RADIUS: f32 = 5.0;
pub const MANA_LABEL_SIZE: f32 = 13.0;
pub const HEALTH_BAR_HEIGHT: f32 = 10.0;
pub const FLOATER_SECONDS: f32 = 0.95;
pub const FLOATER_RISE: f32 = 46.0;
pub const FLOATER_SIZE: f32 = 26.0;
pub const FLOATER_WIDTH: f32 = 80.0;
pub const FLOATER_BOX: Vec2 = Vec2::new(FLOATER_WIDTH, 34.0);
pub const FLOATER_CARD_OFFSET: Vec2 = Vec2::new(MINION_WIDTH * 0.5 - FLOATER_WIDTH * 0.5, -16.0);
pub const SIDE_GAP: f32 = 12.0;
pub const SIDE_OFFSET: f32 = HAND_WIDTH * 0.5 + TABLE_PADDING + SIDE_GAP;
pub const LANE_PADDING: f32 = 10.0;
pub const LANE_BG: Vec4 = Vec4::new(0.0, 0.0, 0.0, 0.22);
pub const LANE_BORDER: Vec4 = Vec4::new(0.78, 0.63, 0.35, 0.12);
pub const TABLE_PADDING: f32 = 32.0;
pub const BANNER_WIDTH: f32 = 760.0;
pub const BANNER_HEIGHT: f32 = 50.0;
pub const BANNER_GAP: f32 = 14.0;
pub const TOP_MARGIN: f32 = 16.0;

pub const TITLE_WIDTH: f32 = 580.0;
pub const TITLE_HEIGHT: f32 = 352.0;
pub const LESSON_TOP: f32 = 112.0;
pub const LESSON_HEIGHT: f32 = 312.0;
pub const LESSON_PORTRAIT: f32 = 52.0;
pub const LESSON_TEXT_HEIGHT: f32 = 152.0;
pub const LESSON_TEXT_SIZE: f32 = 13.5;
pub const BORDER_GUIDE: Vec4 = Vec4::new(1.0, 0.62, 0.12, 1.0);
pub const ECHO_SIGIL: char = '\u{e0ba}';
pub const STRAW_SIGIL: char = '\u{e39e}';
pub const PAUSE_WIDTH: f32 = 420.0;
pub const PAUSE_HEIGHT: f32 = 288.0;
pub const SCRIM: Vec4 = Vec4::new(0.015, 0.02, 0.035, 0.72);
pub const OVER_WIDTH: f32 = 500.0;
pub const OVER_HEIGHT: f32 = 306.0;
pub const SHOP_WIDTH: f32 = 1180.0;
pub const SHOP_HEIGHT: f32 = 516.0;
pub const OFFER_WIDTH: f32 = 210.0;
pub const OFFER_HEIGHT: f32 = 326.0;
pub const OFFER_COUNT: usize = 5;
pub const SCENE_WIDTH: f32 = 880.0;
pub const SCENE_HEIGHT: f32 = 436.0;
pub const SCENE_HEIGHT_FORK: f32 = 558.0;
pub const SCENE_LINE_SIZE: f32 = 17.0;

pub const NODE_SIZE: f32 = 52.0;
pub const NODE_BOX_WIDTH: f32 = 68.0;
pub const NODE_LABEL_HEIGHT: f32 = 18.0;
pub const NODE_ROW_GAP: f32 = 2.0;
pub const NODE_BOX_HEIGHT: f32 = NODE_SIZE + NODE_ROW_GAP + NODE_LABEL_HEIGHT;
pub const COLUMN_PITCH: f32 = 74.0;
pub const LANE_PITCH: f32 = 82.0;
pub const ROAD_DOT: f32 = 5.0;
pub const ROAD_DOTS: usize = 4;
pub const ROAD_DIM: Vec4 = Vec4::new(0.72, 0.62, 0.42, 0.34);
pub const MAP_AREA_WIDTH: f32 = NODE_BOX_WIDTH + (COLUMNS as f32 - 1.0) * COLUMN_PITCH;
pub const MAP_AREA_HEIGHT: f32 = NODE_BOX_HEIGHT + (LANES as f32 - 1.0) * LANE_PITCH;
pub const MAP_WIDTH: f32 = MAP_AREA_WIDTH + 56.0;
pub const MAP_HEIGHT: f32 = 452.0;
pub const MAP_NAME_SIZE: f32 = 10.5;
pub const MARK_AHEAD: Vec4 = Vec4::new(0.085, 0.095, 0.14, 0.92);
pub const MARK_CLEARED: Vec4 = Vec4::new(0.06, 0.12, 0.095, 0.92);
pub const MARK_HERE: Vec4 = Vec4::new(0.28, 0.21, 0.08, 0.95);
pub const MARK_BORDER: Vec4 = Vec4::new(0.78, 0.63, 0.35, 0.24);

pub const SIDE_TOP: f32 = 20.0;
pub const STRIP_WIDTH: f32 = HERO_PANEL_WIDTH;
pub const STRIP_HEIGHT: f32 = 100.0;
pub const LOG_TOP: f32 = SIDE_TOP + STRIP_HEIGHT + 12.0;
pub const RELIC_SLOT_WIDTH: f32 = HERO_PANEL_WIDTH;
pub const RELIC_SLOT_HEIGHT: f32 = 68.0;
pub const RELIC_SLOT_GAP: f32 = 8.0;

pub const TITLE_SIZE: f32 = 44.0;
pub const HEADING_SIZE: f32 = 30.0;
pub const BANNER_SIZE: f32 = 24.0;
pub const LABEL_SIZE: f32 = 18.0;
pub const SMALL_SIZE: f32 = 14.0;
pub const BUTTON_SIZE: f32 = 18.0;
pub const CARD_NAME_SIZE: f32 = 11.5;
pub const CARD_TEXT_SIZE: f32 = 9.5;
pub const MINION_NAME_SIZE: f32 = 10.0;
pub const STAT_SIZE: f32 = 16.0;
pub const ICON_SIZE: f32 = 28.0;
pub const MINION_ICON_SIZE: f32 = 22.0;
pub const BADGE_SIZE: f32 = 22.0;
pub const MINION_BADGE_SIZE: f32 = 20.0;
pub const MINION_BADGE_INSET: f32 = 13.0;
pub const CARD_NAME_INSET: f32 = 6.0;
pub const CARD_TEXT_INSET: f32 = 8.0;
pub const HERO_HEALTH_SIZE: f32 = 26.0;

pub const BATTLE_EXTENT: Vec2 = Vec2::new(
    (SIDE_OFFSET + HERO_PANEL_WIDTH) * 2.0 + 48.0,
    TOP_MARGIN
        + ENEMY_PANEL_HEIGHT
        + BANNER_GAP
        + BANNER_HEIGHT
        + 16.0
        + ARENA_HEIGHT
        + LANE_PADDING
        + HAND_BOTTOM_MARGIN,
);

pub const TITLE_EXTENT: Vec2 = Vec2::new(TITLE_WIDTH + 200.0, TITLE_HEIGHT + 200.0);
pub const SCENE_EXTENT: Vec2 = Vec2::new(1180.0, SCENE_HEIGHT_FORK + 160.0);
pub const MAP_EXTENT: Vec2 = Vec2::new(MAP_WIDTH + 120.0, MAP_HEIGHT + 260.0);
pub const SHOP_EXTENT: Vec2 = Vec2::new(SHOP_WIDTH + 120.0, SHOP_HEIGHT + 240.0);
pub const OVER_EXTENT: Vec2 = Vec2::new(OVER_WIDTH + 200.0, OVER_HEIGHT + 200.0);
pub const MIN_UI_SCALE: f32 = 0.55;
pub const MAX_UI_SCALE: f32 = 1.6;
pub const WINDOW_START_SIZE: (u32, u32) = (1440, 810);
pub const WINDOW_MIN_SIZE: (u32, u32) = (880, 560);

pub const AI_THINK_SECONDS: f32 = 0.55;
pub const OUTCOME_HOLD_SECONDS: f32 = 1.6;
pub const BANNER_SECONDS: f32 = 1.4;
pub const BURST_SECONDS: f32 = 2.0;
pub const BURST_DISTANCE: f32 = 7.0;
pub const CAMERA_FOV_RADIANS: f32 = std::f32::consts::FRAC_PI_4;
pub const CAMERA_EYE: Vec3 = Vec3::new(0.0, 0.0, 0.0);
pub const CAMERA_TARGET: Vec3 = Vec3::new(0.0, 0.0, -1.0);

pub const FRAME_RATE_LIMIT: Option<f32> =
    if cfg!(any(target_os = "android", target_arch = "wasm32")) {
        Some(60.0)
    } else {
        None
    };

pub const SHIELD_BADGE_SIZE: f32 = 18.0;
pub const SHIELD_BADGE_FILL: Vec4 = Vec4::new(0.08, 0.18, 0.28, 0.92);

pub const SMALL_DEVICE: bool = cfg!(any(target_os = "android", target_arch = "wasm32"));
pub const FOREST_OCTAVES: f32 = if SMALL_DEVICE { 3.0 } else { 5.0 };
pub const FOREST_MIST: f32 = if SMALL_DEVICE { 0.0 } else { 1.0 };
pub const FOREST_FLIES: f32 = if SMALL_DEVICE { 0.0 } else { 1.0 };
pub const FOREST_DITHER: f32 = 72.0;
