use nalgebra_glm::Vec3;

pub const TITLE: &str = "Composed";

pub const MEADOW_REACH: f32 = 26.0;
pub const GRAZERS: usize = 14;
pub const GRAZER_HEIGHT: f32 = 0.5;
pub const GRAZER_SCALE: Vec3 = Vec3::new(0.7, 1.0, 0.7);
pub const GRAZER_TINT: [f32; 4] = [0.72, 0.58, 0.36, 1.0];
pub const GROUND_TINT: [f32; 4] = [0.18, 0.26, 0.16, 1.0];
pub const STONE_TINT: [f32; 4] = [0.42, 0.42, 0.46, 1.0];

pub const WANDER_REACH: f32 = 9.0;
pub const WANDER_SPEED: f32 = 1.6;
pub const ROAMING: f32 = 1.0;

pub const CAMERA_BACK: Vec3 = Vec3::new(0.0, 9.0, 16.0);
pub const CAMERA_RATE: f32 = 3.5;

pub const STRIKE_REACH: f32 = 7.0;
pub const STRIKE_DAMAGE: f32 = 34.0;
pub const STRIKE_WAIT: f32 = 0.45;
pub const STRIKE_TRAUMA: f32 = 0.55;

pub const GRAZER_HEALTH: f32 = 100.0;
pub const GRAZER_ARMOR: f32 = 4.0;

pub const HOURS_A_SECOND: f32 = 0.6;
pub const START_HOUR: f32 = 8.0;
