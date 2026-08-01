use nalgebra_glm::{Vec3, Vec4};

pub const TITLE: &str = "Workers";

pub const ACROSS: usize = 8;
pub const STANDS: usize = ACROSS * ACROSS;
pub const STAND_SPAN: f32 = 7.0;
pub const STAND_WIDE: f32 = 4.4;
pub const STAND_TALL: f32 = 1.2;

pub const STATUE_RIDE: f32 = 3.6;
pub const STATUE_SIZE: f32 = 2.1;
pub const RINGS: usize = 80;
pub const SEGMENTS: usize = 120;
pub const BUMPS: usize = 24000;
pub const BUMP_WEIGHT: f32 = 0.09;
pub const BUMP_TIGHT: f32 = 26.0;

pub const MARKER_RIDE: f32 = 4.4;
pub const MARKER_SIZE: f32 = 1.3;
pub const MARKER_SPIN: f32 = 2.6;

pub const TAKE_LIMIT: usize = 1;

pub const FLOOR_MESH: &str = "workers_floor";
pub const STAND_MESH: &str = "workers_stand";
pub const MARKER_MESH: &str = "workers_marker";
pub const FLOOR_MATERIAL: &str = "workers_floor";
pub const STAND_MATERIAL: &str = "workers_stand";
pub const MARKER_MATERIAL: &str = "workers_marker";
pub const STATUE_MATERIAL: &str = "workers_statue";

pub const FLOOR_COLOR: [f32; 4] = [0.09, 0.10, 0.13, 1.0];
pub const STAND_COLOR: [f32; 4] = [0.42, 0.44, 0.50, 1.0];
pub const MARKER_COLOR: [f32; 4] = [1.0, 0.66, 0.14, 1.0];
pub const STATUE_COLOR: [f32; 4] = [0.92, 0.92, 0.94, 1.0];
pub const ROUGH: f32 = 0.7;

pub const LOW_STONE: Vec3 = Vec3::new(0.24, 0.30, 0.42);
pub const HIGH_STONE: Vec3 = Vec3::new(0.95, 0.78, 0.52);
pub const TINTS: [Vec3; 6] = [
    Vec3::new(1.0, 0.86, 0.72),
    Vec3::new(0.78, 0.92, 1.0),
    Vec3::new(0.86, 1.0, 0.82),
    Vec3::new(1.0, 0.80, 0.86),
    Vec3::new(0.92, 0.88, 1.0),
    Vec3::new(1.0, 0.95, 0.74),
];

pub const SUN_INTENSITY: f32 = 4.0;
pub const CAMERA_BACK: f32 = 74.0;
pub const CAMERA_YAW: f32 = 0.7;
pub const CAMERA_PITCH: f32 = 0.48;
pub const CAMERA_HEIGHT: f32 = 3.0;
pub const CAMERA_TURN: f32 = 0.12;

pub const PANEL_MARGIN: f32 = 16.0;
pub const PANEL_WIDTH: f32 = 312.0;
pub const PANEL_HEIGHT: f32 = 404.0;
pub const PANEL_PADDING: f32 = 14.0;
pub const PANEL_SPACING: f32 = 8.0;
pub const PANEL_BACKGROUND: Vec4 = Vec4::new(0.05, 0.05, 0.08, 0.86);
pub const PANEL_BORDER: Vec4 = Vec4::new(1.0, 1.0, 1.0, 0.12);
pub const HEADING_SIZE: f32 = 22.0;
pub const READOUT_SIZE: f32 = 17.0;
pub const READOUT_HEIGHT: f32 = 22.0;
pub const HEADING_COLOR: Vec4 = Vec4::new(0.92, 0.94, 1.0, 1.0);
pub const READOUT_COLOR: Vec4 = Vec4::new(0.78, 0.80, 0.86, 1.0);
pub const GOOD_RATE: u32 = 70;
pub const GOOD_COLOR: Vec4 = Vec4::new(0.45, 0.92, 0.50, 1.0);
pub const POOR_COLOR: Vec4 = Vec4::new(1.0, 0.36, 0.36, 1.0);

pub const NOTES_WIDTH: f32 = 330.0;
pub const NOTES_HEIGHT: f32 = 496.0;
pub const NOTE_SIZE: f32 = 15.0;
pub const NOTE_HEIGHT: f32 = 19.0;
pub const NOTE_SPACING: f32 = 1.0;
pub const NOTE_COLOR: Vec4 = Vec4::new(0.74, 0.76, 0.82, 1.0);
pub const LABEL_COLOR: Vec4 = Vec4::new(1.0, 0.78, 0.35, 1.0);

pub const FRAME_LAMPS: usize = 48;
pub const LAMP_HEIGHT: f32 = 11.0;
pub const FRAME_LAMP_WIDTH: f32 = 4.0;
pub const SLOT_LAMP_WIDTH: f32 = 2.6;
pub const SLOT_LAMP_GAP: f32 = 1.0;
pub const LAMP_GAP: f32 = 1.9;
pub const FRAME_STEP: f32 = 8.0;
pub const FRAME_FAIR: f32 = 12.0;
pub const FRAME_POOR: f32 = 26.0;
pub const CLEAR: Vec4 = Vec4::new(0.0, 0.0, 0.0, 0.0);
pub const LAMP_DARK: Vec4 = Vec4::new(0.16, 0.17, 0.21, 1.0);
pub const LAMP_HEAD: Vec4 = Vec4::new(0.95, 0.95, 1.0, 1.0);
pub const LAMP_GOOD: Vec4 = Vec4::new(0.30, 0.68, 0.36, 1.0);
pub const LAMP_FAIR: Vec4 = Vec4::new(0.88, 0.64, 0.22, 1.0);
pub const LAMP_POOR: Vec4 = Vec4::new(0.92, 0.26, 0.22, 1.0);
pub const LAMP_QUEUED: Vec4 = Vec4::new(0.86, 0.62, 0.20, 1.0);
pub const LAMP_HAND: Vec4 = Vec4::new(0.45, 0.92, 0.50, 1.0);
pub const LAMP_STANDING: Vec4 = Vec4::new(0.80, 0.84, 0.92, 1.0);
