pub const TITLE: &str = "Traced bounce";

pub const CAMERA_FOCUS: [f32; 3] = [0.0, 2.0, 0.0];
pub const CAMERA_BACK: f32 = 8.5;
pub const CAMERA_YAW: f32 = 0.0;
pub const CAMERA_PITCH: f32 = 0.02;

pub const BACKDROP: [f32; 4] = [0.01, 0.01, 0.015, 1.0];

pub const REACH: f32 = 2.0;
pub const THICKNESS: f32 = 0.2;

pub const WHITE: [f32; 4] = [0.85, 0.85, 0.82, 1.0];
pub const ROUGHNESS: f32 = 0.95;

pub const LEFT_WALL: [f32; 4] = [0.75, 0.05, 0.03, 1.0];
pub const RIGHT_WALL: [f32; 4] = [0.03, 0.12, 0.75, 1.0];

pub const LAMP_REACH: f32 = 0.8;
pub const LAMP_THICKNESS: f32 = 0.08;
pub const LAMP_LIGHT: [f32; 3] = [4.0, 3.7, 3.2];
pub const LAMP_INTENSITY: f32 = 26.0;
pub const LAMP_RANGE: f32 = 14.0;

pub const EXPOSURE: f32 = 0.6;

pub const BALL_REACH: f32 = 0.8;

pub const PROBE_COUNTS: [u32; 3] = [8, 6, 8];
pub const PROBE_ORIGIN: [f32; 3] = [-1.75, 0.4, -1.75];
pub const PROBE_SPACING: [f32; 3] = [0.5, 0.62, 0.5];
pub const PROBE_INTENSITY: f32 = 1.0;
pub const PROBE_HYSTERESIS: f32 = 0.93;
pub const PROBE_BOUNCE: f32 = 0.9;
pub const PROBE_REACH: f32 = 12.0;
