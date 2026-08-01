use nightshade::prelude::{Vec3, Vec4};

pub const NIGHT_FROM: f32 = 19.0;
pub const NIGHT_UNTIL: f32 = 6.0;

pub const FOG_COLOR: [f32; 3] = [0.05, 0.06, 0.10];
pub const FOG_START: f32 = 40.0;
pub const BLOOM_THRESHOLD: f32 = 0.9;

pub const GROUND_NAME: &str = "gridlock_ground";
pub const GROUND_COLOR: [f32; 4] = [0.07, 0.075, 0.09, 1.0];
pub const GROUND_ROUGHNESS: f32 = 0.32;
pub const PAVEMENT_NAME: &str = "gridlock_pavement";
pub const PAVEMENT_COLOR: [f32; 4] = [0.46, 0.47, 0.5, 1.0];
pub const PAVEMENT_ROUGHNESS: f32 = 0.9;

pub const NEON_NAMES: [&str; 6] = [
    "gridlock_neon_magenta",
    "gridlock_neon_cyan",
    "gridlock_neon_orange",
    "gridlock_neon_violet",
    "gridlock_neon_green",
    "gridlock_neon_white",
];
pub const NEON_COLORS: [[f32; 3]; 6] = [
    [1.0, 0.12, 0.62],
    [0.10, 0.92, 1.0],
    [1.0, 0.52, 0.10],
    [0.62, 0.22, 1.0],
    [0.22, 1.0, 0.42],
    [0.95, 0.95, 1.0],
];
pub const NEON_DEPTH: f32 = 0.18;
pub const NEON_HEIGHT: f32 = 0.35;

pub const LAMP_WARM: Vec3 = Vec3::new(1.0, 0.72, 0.42);
pub const LAMP_COOL: Vec3 = Vec3::new(0.70, 0.86, 1.0);

pub const FLASH_COLORS: [Vec4; 6] = [
    Vec4::new(1.0, 0.2, 0.7, 1.0),
    Vec4::new(0.2, 1.0, 1.0, 1.0),
    Vec4::new(1.0, 0.95, 0.1, 1.0),
    Vec4::new(0.4, 1.0, 0.2, 1.0),
    Vec4::new(1.0, 0.5, 0.1, 1.0),
    Vec4::new(0.8, 0.3, 1.0, 1.0),
];
pub const FLASH_WORDS: [&str; 12] = [
    "",
    "",
    "NICE",
    "BRUTAL",
    "ABSURD",
    "OBSCENE",
    "UNREAL",
    "LUDICROUS",
    "GALACTIC",
    "COSMIC",
    "ILLEGAL",
    "INFINITE",
];
pub const FLASH_BASE: u64 = 100;
pub const FLASH_PROP_BASE: u64 = 25;
pub const FLASH_GROWTH: u64 = 5;
pub const FLASH_POP: f32 = 1.6;
pub const FLASH_POP_STEP: f32 = 0.35;
pub const FLASH_POP_MAX: f32 = 6.0;
pub const FLASH_HEIGHT: f32 = 2.2;
pub const FLASH_JITTER: f32 = 0.8;
pub const FLASH_SINK: f32 = -80.0;
pub const FLASH_OUTLINE: f32 = 2.5;
pub const FLASH_COUNT: usize = 48;

pub const RAIN_REACH: Vec3 = Vec3::new(18.0, 0.4, 18.0);
pub const RAIN_HEIGHT: f32 = 14.0;
pub const RAIN_COLOR: Vec4 = Vec4::new(0.72, 0.80, 1.0, 0.55);
