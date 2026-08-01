use nightshade::prelude::*;

pub const PANEL_DISTANCE: f32 = 2.6;
pub const PANEL_WIDTH: f32 = 4.7;

pub const CYAN: Vec4 = Vec4::new(0.30, 0.86, 1.0, 0.95);
pub const CYAN_DIM: Vec4 = Vec4::new(0.30, 0.86, 1.0, 0.35);
pub const CYAN_FAINT: Vec4 = Vec4::new(0.30, 0.86, 1.0, 0.14);
pub const AMBER: Vec4 = Vec4::new(1.0, 0.62, 0.18, 0.95);
pub const AMBER_DIM: Vec4 = Vec4::new(1.0, 0.62, 0.18, 0.40);
pub const RED: Vec4 = Vec4::new(1.0, 0.25, 0.25, 0.95);
pub const RED_DIM: Vec4 = Vec4::new(1.0, 0.25, 0.25, 0.35);
pub const WHITE: Vec4 = Vec4::new(0.92, 0.97, 1.0, 0.95);
pub const BUTTON_FACE: Vec4 = Vec4::new(0.05, 0.16, 0.24, 0.85);
pub const GLOW: Vec4 = Vec4::new(0.30, 0.86, 1.0, 0.10);
pub const GLOW_CORE: Vec4 = Vec4::new(0.55, 0.92, 1.0, 0.35);

pub const THIN: f32 = 1.5;
pub const LINE: f32 = 2.0;
pub const HEAVY: f32 = 3.0;

pub const TEXT_SMALL: f32 = 12.0;
pub const TEXT_BODY: f32 = 15.0;
pub const TEXT_LARGE: f32 = 22.0;

pub const RETICLE_OUTER: f32 = 112.0;
pub const RETICLE_INNER: f32 = 74.0;
pub const RETICLE_TICKS: f32 = 132.0;
pub const REACTOR_RADIUS: f32 = 44.0;
pub const BRACKET_ARM: f32 = 14.0;
pub const CORNER_ARM: f32 = 46.0;
pub const MARGIN: f32 = 26.0;
pub const RADAR_RADIUS: f32 = 52.0;
pub const RADAR_RANGE: f32 = 14.0;

pub const BOOT_LINES: [(&str, f32); 6] = [
    ("INITIALIZING VISOR", 0.2),
    ("ARC REACTOR ONLINE", 0.9),
    ("FLIGHT SYSTEMS NOMINAL", 1.5),
    ("WEAPONS HOT", 2.1),
    ("HAND TRACKING LOCKED", 2.6),
    ("AT YOUR SERVICE, SIR", 3.4),
];
pub const BOOT_SECONDS: f32 = 3.8;

pub const DRONE_COLOR: [f32; 3] = [1.0, 0.55, 0.15];
pub const GAUNTLET_COLOR: [f32; 4] = [0.55, 0.08, 0.06, 1.0];
pub const GAUNTLET_TRIM: [f32; 4] = [0.85, 0.65, 0.20, 1.0];
pub const REPULSOR_COLOR: [f32; 3] = [0.45, 0.90, 1.0];
pub const FLARE_COLOR: [f32; 3] = [0.60, 0.85, 1.0];
pub const BOLT_COLOR: [f32; 3] = [0.55, 0.95, 1.0];
pub const MISSILE_COLOR: [f32; 3] = [1.0, 0.70, 0.30];
pub const FLOOR_COLOR: [f32; 4] = [0.08, 0.09, 0.11, 1.0];

pub const THRUST: f32 = 14.0;
pub const DRAG: f32 = 0.9;
pub const MAX_SPEED: f32 = 24.0;
pub const SNAP_TURN_RADIANS: f32 = std::f32::consts::FRAC_PI_6;

pub const BOLT_SPEED: f32 = 34.0;
pub const BOLT_LIFE: f32 = 2.5;
pub const BOLT_COOLDOWN: f32 = 0.22;
pub const BOLT_HIT_RADIUS: f32 = 0.55;
pub const MISSILE_SPEED: f32 = 17.0;
pub const MISSILE_TURN: f32 = 5.0;
pub const MISSILE_LIFE: f32 = 7.0;
pub const MISSILE_HIT_RADIUS: f32 = 0.7;
pub const LOCK_INTERVAL: f32 = 0.28;
pub const ARM_DOWN_COSINE: f32 = 0.72;
pub const DRONE_HEALTH: i32 = 3;
pub const DRONE_RESPAWN: f32 = 3.0;
pub const EFFECT_SECONDS: f32 = 3.5;
pub const PROXIMITY_METERS: f32 = 2.2;
