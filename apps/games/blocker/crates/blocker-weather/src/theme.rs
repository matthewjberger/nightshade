use nalgebra_glm::Vec3;

pub const MENU_HOUR: f32 = 12.0;

pub const SKY_DAY: Vec3 = Vec3::new(0.62, 0.76, 0.95);

pub const SKY_DAWN: Vec3 = Vec3::new(0.85, 0.62, 0.5);

pub const SKY_DUSK: Vec3 = Vec3::new(0.72, 0.45, 0.42);

pub const SKY_NIGHT: Vec3 = Vec3::new(0.06, 0.08, 0.16);

pub const FOG_START: f32 = 243.0;

pub const FOG_END: f32 = 315.0;

pub const FOG_HEIGHT: f32 = 90.0;

pub const FOG_FALLOFF: f32 = 34.0;

pub const BLOOM_DAY: f32 = 0.1;

pub const BLOOM_NIGHT: f32 = 0.24;

pub const BLOOM_FLASH: f32 = 0.85;

pub const SKY_OVERCAST: Vec3 = Vec3::new(0.44, 0.46, 0.5);

pub const RAIN_FOG_START: f32 = 120.0;

pub const RAIN_FOG_END: f32 = 260.0;

pub const WEATHER_FIRST: f32 = 45.0;

pub const WEATHER_FADE: f32 = 5.0;

pub const FREEZING: f32 = 0.0;

pub const HARD_FREEZE: f32 = -8.0;

pub const TEMP_EASE: f32 = 0.15;

pub const FOG_NEAR_START: f32 = 30.0;

pub const FOG_NEAR_END: f32 = 120.0;

pub const RAIN_HEIGHT: f32 = 12.0;

pub const RAIN_DEPTH: i32 = 48;

pub const CELL_SIZE: i32 = 5;

pub const CELLS_ACROSS: usize = 7;

pub const CELL_HALF: Vec3 = Vec3::new(2.5, 0.2, 2.5);

pub const SHORTEST_DROP: f32 = 0.5;

pub const RAIN_RATE: f32 = 14.5;

pub const RAIN_SPEED: f32 = 15.0;

pub const SNOW_RATE: f32 = 5.0;

pub const SNOW_SPEED: f32 = 2.0;

pub const FLAT_STEP: i32 = 1;

pub const SPLASH_RATE: f32 = 9.0;

pub const SPLASH_RISE: f32 = 0.06;

pub const WIND_WANDER: f32 = 0.08;

pub const RAIN_SLANT: f32 = 1.0;

pub const SNOW_SLANT: f32 = 2.5;

pub const COVER_SNOW: f32 = 0.3;

pub const COVER_REACH: f32 = 24.0;

pub const FLASH_SECONDS: f32 = 0.25;
