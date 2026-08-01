use nalgebra_glm::{Vec3, Vec4};

pub const BOAT_MESH: &str = "voxels_boat";

pub const BOAT_EASE: f32 = 3.5;

pub const BOAT_FLOAT: f32 = 0.22;

pub const BOAT_REACH: f32 = 2.2;

pub const SWING_SECONDS: f32 = 0.28;

pub const PLAYER_HALF_WIDTH: f32 = 0.3;

pub const PLAYER_HEIGHT: f32 = 1.8;

pub const EYE_HEIGHT: f32 = 1.62;

pub const CROUCH_EYE_HEIGHT: f32 = 1.4;

pub const WATCH_SPEED: f32 = 22.0;

pub const WATCH_SPEED_MIN: f32 = 6.0;

pub const WATCH_SPEED_MAX: f32 = 90.0;

pub const WATCH_SPEED_STEP: f32 = 4.0;

pub const PICKUP_DELAY: f32 = 0.6;

pub const OUTLINE_COLOR: Vec4 = Vec4::new(0.05, 0.05, 0.05, 1.0);

pub const FIREWORK_FUSE: f32 = 1.5;

pub const FIREWORK_SPEED: f32 = 17.0;

pub const FIREWORK_COLORS: [Vec3; 5] = [
    Vec3::new(1.0, 0.35, 0.3),
    Vec3::new(0.4, 0.7, 1.0),
    Vec3::new(1.0, 0.85, 0.35),
    Vec3::new(0.5, 1.0, 0.5),
    Vec3::new(1.0, 0.5, 0.9),
];

pub const ZOOM_START: f32 = 5.5;

pub const ZOOM_MIN: f32 = 0.0;

pub const ZOOM_MAX: f32 = 8.0;

pub const ZOOM_STEP: f32 = 0.6;

pub const CAMERA_COLLIDE_STEP: f32 = 0.2;

pub const CAMERA_COLLIDE_MARGIN: f32 = 0.4;

pub const FIREWORK_PARTICLES: u32 = 220;

pub const FIREWORK_SPARK_SIZE: f32 = 0.12;

pub const FIREWORK_SCALE: f32 = 0.4;

pub const GLIDER_LIFT: f32 = 2.1;

pub const ZOOM_FIRST_PERSON: f32 = 1.2;

pub const FLOAT_GAIT: f32 = 0.35;

pub const SOAR_GAIT: f32 = 0.1;

pub const START_PITCH: f32 = -0.15;

pub const ARRIVAL_RADIUS: i32 = 6;

pub const HAND_BLOCK_OFFSET: Vec3 = Vec3::new(0.34, -0.30, 0.62);

pub const HAND_BLOCK_YAW: f32 = -0.6;

pub const HAND_BLOCK_PITCH: f32 = 0.08;

pub const HAND_BLOCK_SCALE: f32 = 0.24;

pub const HAND_TOOL_OFFSET: Vec3 = Vec3::new(0.36, -0.26, 0.6);

pub const HAND_TOOL_YAW: f32 = std::f32::consts::PI - 0.5;

pub const HAND_TOOL_PITCH: f32 = -0.35;

pub const HAND_TOOL_SCALE: f32 = 0.5;

pub const HAND_DIP: Vec3 = Vec3::new(0.0, -0.12, 0.08);

pub const HAND_SWING_ARC: f32 = -1.1;

pub const BOAT_EXTENT: Vec3 = Vec3::new(1.6, 0.8, 3.2);

pub const GLIDER_MESH: &str = "voxels_glider";

pub const GLIDER_LOW: Vec3 = Vec3::new(-2.5, -0.8, -1.7);

pub const GLIDER_HIGH: Vec3 = Vec3::new(2.5, 0.6, 0.8);

pub const WING_PITCH: f32 = 0.12;

pub const WING_RISE: f32 = 0.35;

pub const WING_BACK: f32 = 0.1;

pub const SHELL_GRAVITY: f32 = 9.81;
