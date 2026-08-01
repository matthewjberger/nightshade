use nalgebra_glm::Vec3;

pub const TITLE: &str = "Senses";
pub const REACH: f32 = 20.0;
pub const FLOOR_TINT: [f32; 4] = [0.16, 0.16, 0.18, 1.0];
pub const HIDER_TINT: [f32; 4] = [0.40, 0.70, 0.55, 1.0];
pub const WATCHER_CALM: [f32; 4] = [0.35, 0.40, 0.55, 1.0];
pub const WATCHER_ALARMED: [f32; 4] = [0.85, 0.25, 0.20, 1.0];
pub const WATCHERS: usize = 3;
pub const WATCHER_RING: f32 = 9.0;
pub const WALK: f32 = 6.0;
pub const CROUCH_WALK: f32 = 2.6;
pub const TURN_RATE: f32 = 0.7;
pub const HEARING: f32 = 12.0;
pub const SIGHT: f32 = 14.0;
pub const FOOTFALL: f32 = 0.45;
pub const STEP_LOUDNESS: f32 = 0.5;
pub const SHOUT_LOUDNESS: f32 = 1.0;
pub const CROUCH_SCALE: f32 = 0.45;
pub const MOVING_SCALE: f32 = 1.4;
pub const HALF_ANGLE: f32 = 0.9;
pub const NOISE_GAIN: f32 = 0.6;
pub const CALM: f32 = 0.35;
pub const ALARMED_ABOVE: f32 = 0.5;

pub const CAMERA_TARGET: Vec3 = Vec3::new(0.0, 0.0, 0.0);
pub const CAMERA_BACK: f32 = 30.0;
pub const CAMERA_YAW: f32 = 0.0;
pub const CAMERA_PITCH: f32 = 1.1;
