use nalgebra_glm::Vec3;

pub const TITLE: &str = "Tiles";
pub const WALL_TINT: [f32; 4] = [0.36, 0.34, 0.40, 1.0];
pub const OPEN_TINT: [f32; 4] = [0.18, 0.22, 0.26, 1.0];
pub const STEP: f32 = 1.4;
pub const WALL: u8 = b'#';
pub const OPEN: u8 = b'.';
pub const CURSOR_TINT: [f32; 4] = [0.95, 0.72, 0.30, 1.0];
pub const CURSOR_HEIGHT: f32 = 1.4;
pub const LIT: f32 = 0.4;
pub const FLAT: f32 = 0.1;

pub const MAP: [&str; 9] = [
    "#########",
    "#...#...#",
    "#.#.#.#.#",
    "#.#...#.#",
    "#.#####.#",
    "#.......#",
    "#.#####.#",
    "#.......#",
    "#########",
];

pub const CAMERA_TARGET: Vec3 = Vec3::new(0.0, 0.0, 0.0);
pub const CAMERA_BACK: f32 = 20.0;
pub const CAMERA_YAW: f32 = 0.0;
pub const CAMERA_PITCH: f32 = 1.1;
