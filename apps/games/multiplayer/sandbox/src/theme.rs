use nalgebra_glm::Vec4;

pub const TITLE: &str = "Multiplayer Sandbox";

pub const DEFAULT_PORT: u16 = 7777;
pub const LOOPBACK: &str = "127.0.0.1";

pub const TICK_HZ: f32 = 30.0;
pub const TICK_SECONDS: f32 = 1.0 / TICK_HZ;
pub const INTERPOLATION_DELAY: f32 = 0.1;
pub const TRACK_DEPTH: usize = 24;
pub const SAVED_INPUTS: usize = 64;
pub const CORRECTION_BLEND: f32 = 0.25;
pub const CORRECTION_SNAP: f32 = 2.0;
pub const CHAT_LINES: usize = 6;
pub const ROSTER_LINES: usize = 4;
pub const CHAT_LIMIT: usize = 120;
pub const HISTORY_TICKS: usize = 64;
pub const INTEREST_RADIUS: f32 = 30.0;

pub const TAG_RANGE: f32 = 60.0;
pub const TAG_RADIUS: f32 = 0.6;
pub const TAG_LOW: f32 = 0.2;
pub const TAG_HIGH: f32 = 1.8;
pub const MARK_SECONDS: f32 = 8.0;
pub const MARK_SCALE: f32 = 0.6;

pub const BUMPER_RADIUS: f32 = 0.55;
pub const BUMPER_HALF_HEIGHT: f32 = 0.55;
pub const BUMPER_CENTER_HEIGHT: f32 = 1.55;
pub const LIFTER_RADIUS: f32 = 0.45;
pub const RIDE_PROBE_HEIGHT: f32 = 1.0;
pub const LIFTER_PROBE_DISTANCE: f32 = 0.4;
pub const LIFTER_SPRING_STIFFNESS: f32 = 200.0;
pub const LIFTER_SPRING_DAMPING: f32 = 22.0;
pub const GROUNDED_MARGIN: f32 = 0.15;
pub const MAX_GROUND_SPEED: f32 = 5.5;
pub const GROUND_ACCELERATION: f32 = 40.0;
pub const AIR_ACCELERATION: f32 = 12.0;
pub const GROUND_FRICTION: f32 = 8.0;
pub const JUMP_IMPULSE: f32 = 5.5;
pub const JUMP_SUPPRESS_SECONDS: f32 = 0.3;
pub const CROUCH_SHARPNESS: f32 = 12.0;
pub const CROUCH_SPEED_PENALTY: f32 = 0.65;
pub const FACING_SPEED_THRESHOLD: f32 = 0.5;
pub const FACING_TURN_SHARPNESS: f32 = 12.0;
pub const MAX_STEP_SECONDS: f32 = 1.0 / 30.0;
pub const BUMPER_RESOLVE_ITERATIONS: usize = 3;
pub const FALL_FLOOR: f32 = -40.0;

pub const CAMERA_DISTANCE: f32 = 6.0;
pub const CAMERA_PITCH: f32 = 0.3;
pub const CAMERA_FOV_DEGREES: f32 = 70.0;

pub const WALK_SPEED: f32 = 1.6;
pub const RUN_SPEED: f32 = 4.5;
pub const MODEL_FACING: f32 = std::f32::consts::PI;
pub const CHARACTER_SCALE: f32 = 1.0;
pub const ANIMAL_SCALE: f32 = 1.0;

pub const WANDER_TURN: f32 = 1.8;
pub const SPAWN_RING: f32 = 4.0;
pub const GROUND_EXTENT: f32 = 400.0;
pub const GROUND_COLOR: [f32; 4] = [0.34, 0.40, 0.30, 1.0];

pub const PANEL_BACKGROUND: Vec4 = Vec4::new(0.06, 0.07, 0.10, 0.88);
pub const PANEL_BORDER: Vec4 = Vec4::new(0.30, 0.34, 0.44, 1.0);
pub const TEXT_PRIMARY: Vec4 = Vec4::new(0.90, 0.93, 1.0, 1.0);
pub const TEXT_DIM: Vec4 = Vec4::new(0.58, 0.63, 0.74, 1.0);
pub const PANEL_TEXT_SIZE: f32 = 15.0;
pub const HEADING_TEXT_SIZE: f32 = 19.0;
