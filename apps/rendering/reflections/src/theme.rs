pub const TITLE: &str = "Reflections";

pub const CAMERA_FOCUS: [f32; 3] = [0.0, 0.9, 0.0];
pub const CAMERA_BACK: f32 = 9.0;
pub const CAMERA_YAW: f32 = -0.35;
pub const CAMERA_PITCH: f32 = 0.10;

pub const SKY: &[u8] = include_bytes!("../../../../assets/sky/winter_river_1k.hdr");

pub const FLOOR_REACH: f32 = 14.0;
pub const FLOOR_THICKNESS: f32 = 0.05;
pub const FLOOR_COLOR: [f32; 4] = [0.02, 0.02, 0.02, 1.0];
pub const FLOOR_ROUGHNESS: f32 = 0.04;

pub const BALLS: [(f32, [f32; 4], f32, f32); 5] = [
    (-3.0, [0.85, 0.20, 0.18, 1.0], 0.05, 1.0),
    (-1.5, [0.20, 0.75, 0.30, 1.0], 0.20, 1.0),
    (0.0, [0.20, 0.35, 0.90, 1.0], 0.40, 0.6),
    (1.5, [0.90, 0.80, 0.15, 1.0], 0.65, 0.3),
    (3.0, [0.85, 0.85, 0.85, 1.0], 0.90, 0.0),
];
pub const BALL_HEIGHT: f32 = 0.9;
pub const BALL_REACH: f32 = 0.9;
pub const BALL_ROW: f32 = -2.2;

pub const LAMPS: [(f32, f32, [f32; 3]); 3] = [
    (-4.0, 2.4, [6.0, 1.2, 1.0]),
    (0.0, 3.2, [1.0, 6.0, 2.0]),
    (4.0, 2.4, [1.0, 1.8, 6.0]),
];
pub const LAMP_REACH: f32 = 0.35;
pub const LAMP_ROW: f32 = 2.6;

pub const REFLECTION_STRENGTH: f32 = 1.0;
pub const REFLECTION_REACH: f32 = 50.0;
