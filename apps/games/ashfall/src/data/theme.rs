use crate::data::rooms::RoomKind;
use nightshade::prelude::*;

pub const TITLE: &str = "Ashfall";

pub const PANEL_BG: Vec4 = Vec4::new(0.022, 0.021, 0.024, 0.95);
pub const PANEL_BORDER: Vec4 = Vec4::new(0.42, 0.19, 0.11, 0.85);
pub const PANEL_RULE: Vec4 = Vec4::new(0.32, 0.14, 0.08, 0.7);
pub const BUTTON_BG: Vec4 = Vec4::new(0.055, 0.05, 0.05, 1.0);
pub const BUTTON_HOVER: Vec4 = Vec4::new(0.16, 0.08, 0.055, 1.0);
pub const BUTTON_PRESSED: Vec4 = Vec4::new(0.33, 0.14, 0.08, 1.0);
pub const BUTTON_HEIGHT: f32 = 40.0;
pub const TEXT_PRIMARY: Vec4 = Vec4::new(0.86, 0.82, 0.76, 1.0);
pub const TEXT_DIM: Vec4 = Vec4::new(0.55, 0.52, 0.48, 1.0);
pub const TEXT_RUST: Vec4 = Vec4::new(0.78, 0.36, 0.22, 1.0);
pub const TEXT_OUTLINE: Vec4 = Vec4::new(0.0, 0.0, 0.0, 1.0);
pub const TRANSPARENT: Vec4 = Vec4::new(0.0, 0.0, 0.0, 0.0);
pub const HURT_FLASH: Vec4 = Vec4::new(0.55, 0.02, 0.0, 0.55);
pub const SHIFT_FLASH: Vec4 = Vec4::new(0.9, 0.85, 0.8, 0.9);
pub const MUZZLE_FLASH: Vec4 = Vec4::new(1.0, 0.9, 0.6, 0.35);
pub const BLAST_FLASH: Vec4 = Vec4::new(1.0, 0.62, 0.25, 0.8);

pub const CONDITION_FINE: Vec4 = Vec4::new(0.55, 0.75, 0.5, 1.0);
pub const CONDITION_CAUTION: Vec4 = Vec4::new(0.9, 0.7, 0.3, 1.0);
pub const CONDITION_DANGER: Vec4 = Vec4::new(0.85, 0.15, 0.1, 1.0);
pub const BATTERY_LOW_COLOR: Vec4 = Vec4::new(0.85, 0.5, 0.15, 1.0);

pub const TITLE_WIDTH: f32 = 620.0;
pub const TITLE_HEIGHT: f32 = 330.0;
pub const MENU_WIDTH: f32 = 340.0;
pub const MENU_HEIGHT: f32 = 230.0;
pub const NOTE_WIDTH: f32 = 620.0;
pub const NOTE_HEIGHT: f32 = 360.0;
pub const ENDING_WIDTH: f32 = 620.0;
pub const ENDING_HEIGHT: f32 = 300.0;
pub const INVENTORY_WIDTH: f32 = 620.0;
pub const INVENTORY_HEIGHT: f32 = 620.0;

pub const TITLE_SIZE: f32 = 44.0;
pub const HEADING_SIZE: f32 = 30.0;
pub const HUD_SIZE: f32 = 20.0;
pub const LABEL_SIZE: f32 = 17.0;
pub const NOTE_SIZE: f32 = 18.0;
pub const ROW_HEIGHT: f32 = 24.0;
pub const MAP_TOP: f32 = 56.0;

pub const INSPECT_ORIGIN: Vec3 = Vec3::new(0.0, -600.0, 0.0);
pub const INSPECT_BACK: f32 = 1.6;
pub const INSPECT_SCALE: f32 = 1.6;
pub const INSPECT_LIGHT: Vec3 = Vec3::new(1.0, 0.94, 0.86);
pub const INSPECT_INTENSITY: f32 = 46.0;

pub const CHART_ORIGIN: Vec3 = Vec3::new(0.0, -400.0, 0.0);
pub const CHART_HEIGHT: f32 = 40.0;
pub const CHART_ZOOM: f32 = 27.0;
pub const CHART_PLATE: f32 = 0.1;
pub const CHART_GAP: f32 = 0.35;
pub const CHART_MARK: f32 = 0.8;
pub const CHART_MARK_LIFT: f32 = 0.12;
pub const CHART_MARKER: f32 = 1.3;
pub const CHART_MARKER_LIFT: f32 = 0.24;
pub const CHART_UPPER_LEVEL: f32 = 1.6;
pub const CHART_UNKNOWN: [f32; 4] = [0.07, 0.07, 0.08, 1.0];
pub const CHART_STREET: [f32; 4] = [0.22, 0.22, 0.24, 1.0];
pub const CHART_INSIDE: [f32; 4] = [0.42, 0.40, 0.36, 1.0];
pub const CHART_SPECIAL: [f32; 4] = [0.30, 0.34, 0.40, 1.0];
pub const CHART_OPEN: [f32; 4] = [0.34, 0.30, 0.24, 1.0];
pub const CHART_DOOR: [f32; 4] = [0.70, 0.66, 0.58, 1.0];
pub const CHART_LOCKED: [f32; 4] = [0.78, 0.52, 0.16, 1.0];
pub const CHART_BOLTED: [f32; 4] = [0.62, 0.20, 0.20, 1.0];
pub const CHART_STAIR: [f32; 4] = [0.40, 0.62, 0.66, 1.0];
pub const CHART_GATE: [f32; 4] = [0.85, 0.82, 0.40, 1.0];
pub const CHART_HERE: [f32; 4] = [0.90, 0.30, 0.16, 1.0];

pub fn chart_room_color(kind: RoomKind) -> [f32; 4] {
    match kind {
        RoomKind::Street => CHART_STREET,
        RoomKind::Open => CHART_OPEN,
        RoomKind::Gate | RoomKind::Pharmacy | RoomKind::Cells => CHART_SPECIAL,
        _ => CHART_INSIDE,
    }
}

pub const MAP_WASH: Vec4 = Vec4::new(0.0, 0.0, 0.0, 0.0);
pub const MAP_WALL: Vec4 = Vec4::new(0.62, 0.58, 0.52, 1.0);

pub const FOG_COLOR: [f32; 3] = [0.075, 0.078, 0.085];
pub const FOG_START: f32 = 1.5;
pub const FOG_END: f32 = 24.0;
pub const OTHERWORLD_FOG_COLOR: [f32; 3] = [0.11, 0.02, 0.012];
pub const OTHERWORLD_FOG_START: f32 = 1.0;
pub const OTHERWORLD_FOG_END: f32 = 15.0;
pub const GRADING_SATURATION: f32 = 0.7;
pub const GRADING_CONTRAST: f32 = 1.12;

pub const MOON_COLOR: Vec3 = Vec3::new(0.45, 0.5, 0.68);
pub const MOON_INTENSITY: f32 = 0.55;
pub const OTHERWORLD_MOON_COLOR: Vec3 = Vec3::new(0.7, 0.2, 0.12);
pub const FLASHLIGHT_COLOR: Vec3 = Vec3::new(1.0, 0.93, 0.8);
pub const FLASHLIGHT_INTENSITY: f32 = 210.0;
pub const FLASHLIGHT_RANGE: f32 = 30.0;
pub const FLASHLIGHT_INNER: f32 = 0.20;
pub const FLASHLIGHT_OUTER: f32 = 0.46;
pub const FLASHLIGHT_SHADOW_BIAS: f32 = 0.0004;
pub const FLASHLIGHT_SHADOW_SOFTNESS: f32 = 0.03;
pub const WINDOW_GLOW: [f32; 3] = [0.5, 0.34, 0.15];
pub const WINDOW_RED: [f32; 3] = [0.55, 0.06, 0.02];
pub const WINDOW_LIGHT: Vec3 = Vec3::new(1.0, 0.76, 0.45);
pub const WINDOW_INTENSITY: f32 = 13.0;
pub const WINDOW_RANGE: f32 = 7.0;
pub const WINDOW_LIT_CHANCE: f32 = 0.13;
pub const LAMP_COLOR: Vec3 = Vec3::new(1.0, 0.72, 0.45);
pub const LAMP_INTENSITY: f32 = 90.0;
pub const LAMP_RANGE: f32 = 20.0;
pub const OTHERWORLD_LAMP_COLOR: Vec3 = Vec3::new(1.0, 0.2, 0.08);
pub const LANTERN_INTENSITY: f32 = 58.0;
pub const LANTERN_RANGE: f32 = 13.0;
pub const LANTERN_SHADOW_BIAS: f32 = 0.0025;
pub const LANTERN_SHADOW_SOFTNESS: f32 = 0.5;
pub const START_FACING: f32 = std::f32::consts::FRAC_PI_2;

pub const MENU_CAMERA_EYE: Vec3 = Vec3::new(63.0, 4.5, 88.0);
pub const MENU_CAMERA_TARGET: Vec3 = Vec3::new(63.0, 2.2, 32.0);

pub const ASH_HALF_EXTENTS: Vec3 = Vec3::new(11.0, 4.0, 11.0);
pub const ASH_HEIGHT: f32 = 4.2;
pub const ASH_RATE: f32 = 150.0;
pub const ASH_SIZE_START: f32 = 0.055;
pub const ASH_SIZE_END: f32 = 0.04;
pub const ASH_FALL: f32 = -1.2;
pub const ASH_LIFE: (f32, f32) = (4.5, 8.0);
pub const ASH_COLOR: Vec4 = Vec4::new(0.78, 0.78, 0.8, 0.75);
pub const EMBER_COLOR: Vec4 = Vec4::new(1.0, 0.35, 0.12, 0.75);

pub const STAIN_TEXTURE: &str = "ashfall_stain";
pub const STAIN_SIZE: f32 = 1.9;
pub const STAIN_COLOR: [f32; 4] = [0.30, 0.02, 0.02, 0.92];
pub const SCORCH_COLOR: [f32; 4] = [0.05, 0.04, 0.04, 0.95];
pub const SCORCH_SIZE: f32 = 4.2;
pub const SHARD_COUNT: u32 = 26;
pub const SHARD_COLOR: Vec4 = Vec4::new(0.72, 0.82, 0.85, 0.9);
pub const PUFF_LIFE: f32 = 2.5;

pub const KEY_COLOR: [f32; 4] = [0.85, 0.7, 0.3, 1.0];
pub const KEY_GLOW: [f32; 3] = [0.6, 0.45, 0.12];
pub const WARD_COLOR: [f32; 4] = [0.55, 0.72, 0.7, 1.0];
pub const WARD_GLOW: [f32; 3] = [0.16, 0.4, 0.38];
pub const BONE_COLOR: [f32; 4] = [0.9, 0.86, 0.75, 1.0];
pub const BONE_GLOW: [f32; 3] = [0.35, 0.32, 0.25];
pub const HEALTH_COLOR: [f32; 4] = [0.7, 0.08, 0.06, 1.0];
pub const HEALTH_GLOW: [f32; 3] = [0.5, 0.03, 0.02];
pub const AMMO_COLOR: [f32; 4] = [0.35, 0.33, 0.3, 1.0];
pub const AMMO_GLOW: [f32; 3] = [0.12, 0.11, 0.09];
pub const BATTERY_COLOR: [f32; 4] = [0.2, 0.5, 0.25, 1.0];
pub const BATTERY_GLOW: [f32; 3] = [0.08, 0.3, 0.12];
pub const MAP_ITEM_COLOR: [f32; 4] = [0.8, 0.76, 0.6, 1.0];
pub const MAP_ITEM_GLOW: [f32; 3] = [0.22, 0.2, 0.14];
pub const CASE_COLOR: [f32; 4] = [0.3, 0.31, 0.34, 1.0];
pub const CASE_GLOW: [f32; 3] = [0.09, 0.09, 0.1];
pub const LOCKBOX_COLOR: [f32; 4] = [0.36, 0.34, 0.3, 1.0];
pub const LOCKBOX_GLOW: [f32; 3] = [0.1, 0.09, 0.08];
pub const NOTE_COLOR: [f32; 4] = [0.85, 0.8, 0.7, 1.0];
pub const NOTE_GLOW: [f32; 3] = [0.25, 0.22, 0.18];
pub const HUSK_TINT: [f32; 4] = [0.55, 0.42, 0.38, 1.0];
pub const CRAWLER_TINT: [f32; 4] = [0.42, 0.36, 0.33, 1.0];
pub const WRETCH_TINT: [f32; 4] = [0.4, 0.34, 0.3, 1.0];
pub const SCREAMER_TINT: [f32; 4] = [0.62, 0.5, 0.44, 1.0];
pub const OTHERWORLD_HUSK_TINT: [f32; 4] = [0.5, 0.12, 0.08, 1.0];
pub const PERSON_COAT: [f32; 4] = [0.5, 0.52, 0.6, 1.0];
pub const PERSON_SKIN: [f32; 4] = [0.72, 0.62, 0.55, 1.0];
pub const PERSON_REACH: f32 = 2.8;
pub const PERSON_TURN: f32 = 4.0;
pub const LAMP_HEAD_GLOW: [f32; 3] = [2.5, 1.7, 0.9];

pub fn phase_looks() -> Vec<nightshade_phase::PhaseLook> {
    vec![
        nightshade_phase::PhaseLook {
            fog_color: FOG_COLOR,
            fog_start: FOG_START,
            fog_end: FOG_END,
            saturation: GRADING_SATURATION,
            contrast: GRADING_CONTRAST,
            light_color: LAMP_COLOR,
        },
        nightshade_phase::PhaseLook {
            fog_color: OTHERWORLD_FOG_COLOR,
            fog_start: OTHERWORLD_FOG_START,
            fog_end: OTHERWORLD_FOG_END,
            saturation: GRADING_SATURATION,
            contrast: GRADING_CONTRAST,
            light_color: OTHERWORLD_LAMP_COLOR,
        },
    ]
}
