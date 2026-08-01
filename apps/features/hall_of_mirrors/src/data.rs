use nightshade::prelude::Vec3;

pub const TITLE: &str = "Hall of Mirrors";

pub const LEVELS: usize = 5;

pub const HALL_HALF_WIDTH: f32 = 4.5;
pub const HALL_HALF_LENGTH: f32 = 11.0;
pub const HALL_HEIGHT: f32 = 5.0;
pub const WALL_THICKNESS: f32 = 0.6;

pub const START: [f32; 3] = [0.0, 0.0, 6.0];
pub const START_YAW: f32 = 0.0;
pub const EYE_HEIGHT: f32 = 1.65;
pub const PLAYER_HALF_WIDTH: f32 = 0.35;
pub const PLAYER_HEIGHT: f32 = 1.8;
pub const WALK_SPEED: f32 = 3.6;
pub const GRAVITY: f32 = 20.0;
pub const TERMINAL_FALL: f32 = 42.0;
pub const STEP_LIMIT: f32 = 0.2;
pub const MAX_SLICES: u32 = 24;
pub const MOUSE_SENSITIVITY: f32 = 0.0022;
pub const PITCH_LIMIT: f32 = 1.5;
pub const STICK_LOOK_SPEED: f32 = 2.6;

pub const TRAVEL_MARGIN: f32 = 0.18;
pub const TRAVEL_HOLD: f32 = 0.35;
pub const TRAVEL_CLEARANCE: f32 = 0.32;
pub const TRAVEL_STEPS: u32 = 14;
pub const TRAVEL_STEP: f32 = 0.18;
pub const TRAVEL_HEADING: f32 = 0.2;

pub const FIELD_OF_VIEW: f32 = 1.2;
pub const NEAR_PLANE: f32 = 0.05;
pub const FAR_PLANE: f32 = 400.0;
pub const ARCH_DEPTH: f32 = 0.7;

pub const PORTAL_WIDTH: f32 = 2.6;
pub const PORTAL_HEIGHT: f32 = 3.6;
pub const PORTAL_LIFT: f32 = 1.95;
pub const PORTAL_SURFACE_OFFSET: f32 = 0.06;
pub const PORTAL_TARGET_MAX: u32 = 1400;

pub const REACH: f32 = 90.0;
pub const MENU_WIDTH: f32 = 340.0;
pub const MENU_HEIGHT: f32 = 198.0;
pub const AVATAR_LAYER: u32 = 2;
pub const BLUE_ARCH_LAYER: u32 = 4;
pub const ORANGE_ARCH_LAYER: u32 = 8;
pub const AVATAR_RADIUS: f32 = 0.35;
pub const AVATAR_COLOR: [f32; 4] = [0.86, 0.88, 0.94, 1.0];

pub const NEAR_RIM: [f32; 4] = [0.30, 0.68, 1.0, 1.0];
pub const FAR_RIM: [f32; 4] = [1.0, 0.52, 0.22, 1.0];

const FLOOR_DARK: [f32; 4] = [0.13, 0.13, 0.16, 1.0];
const FLOOR_LIGHT: [f32; 4] = [0.62, 0.60, 0.56, 1.0];
const WALL: [f32; 4] = [0.24, 0.23, 0.28, 1.0];
const TRIM: [f32; 4] = [0.72, 0.58, 0.28, 1.0];
const PILLAR: [f32; 4] = [0.34, 0.33, 0.38, 1.0];
const CEILING: [f32; 4] = [0.10, 0.10, 0.13, 1.0];

pub struct Slab {
    pub center: [f32; 3],
    pub size: [f32; 3],
    pub color: [f32; 4],
    pub emissive: f32,
    pub solid: bool,
}

pub struct Hit {
    pub point: Vec3,
    pub normal: Vec3,
    pub distance: f32,
}

fn slab(center: [f32; 3], size: [f32; 3], color: [f32; 4]) -> Slab {
    Slab {
        center,
        size,
        color,
        emissive: 0.0,
        solid: true,
    }
}

fn tile(center: [f32; 3], size: [f32; 3], color: [f32; 4]) -> Slab {
    Slab {
        center,
        size,
        color,
        emissive: 0.0,
        solid: false,
    }
}

fn lamp(center: [f32; 3], size: [f32; 3], color: [f32; 4]) -> Slab {
    Slab {
        center,
        size,
        color,
        emissive: 5.0,
        solid: false,
    }
}

pub fn hall() -> Vec<Slab> {
    let width = HALL_HALF_WIDTH;
    let length = HALL_HALF_LENGTH;
    let height = HALL_HEIGHT;
    let thickness = WALL_THICKNESS;
    let mut blocks = Vec::new();

    blocks.push(slab(
        [0.0, -thickness * 0.5, 0.0],
        [width * 2.0, thickness, length * 2.0],
        FLOOR_DARK,
    ));
    let columns = 9;
    let span = width * 2.0 / columns as f32;
    let rows = (length * 2.0 / span).ceil() as i32;
    for row in 0..rows {
        for column in 0..columns {
            if (row + column) % 2 != 0 {
                continue;
            }
            blocks.push(tile(
                [
                    -width + span * (column as f32 + 0.5),
                    0.015,
                    -length + span * (row as f32 + 0.5),
                ],
                [span, 0.03, span],
                FLOOR_LIGHT,
            ));
        }
    }

    blocks.push(slab(
        [0.0, height + thickness * 0.5, 0.0],
        [width * 2.0, thickness, length * 2.0],
        CEILING,
    ));
    for side in [-1.0_f32, 1.0] {
        blocks.push(slab(
            [side * (width + thickness * 0.5), height * 0.5, 0.0],
            [thickness, height, length * 2.0],
            WALL,
        ));
        blocks.push(slab(
            [0.0, height * 0.5, side * (length + thickness * 0.5)],
            [width * 2.0, height, thickness],
            WALL,
        ));
        blocks.push(slab(
            [side * width, 0.25, 0.0],
            [0.24, 0.5, length * 2.0],
            TRIM,
        ));
    }

    for step in 0..5 {
        let z = -length + 2.6 + step as f32 * (length * 2.0 - 5.2) / 4.0;
        for side in [-1.0_f32, 1.0] {
            blocks.push(slab(
                [side * (width - 0.45), height * 0.5, z],
                [0.7, height, 0.7],
                PILLAR,
            ));
            blocks.push(lamp(
                [side * (width - 0.45), height - 1.1, z],
                [0.34, 0.5, 0.34],
                [1.0, 0.86, 0.62, 1.0],
            ));
        }
    }

    blocks.push(lamp(
        [0.0, height - 0.35, 0.0],
        [1.4, 0.22, 1.4],
        [0.92, 0.94, 1.0, 1.0],
    ));

    blocks
}

pub fn arch(center: Vec3) -> Vec<Slab> {
    let half = PORTAL_WIDTH * 0.5 + 0.45;
    let top = PORTAL_LIFT + PORTAL_HEIGHT * 0.5 + 0.4;
    let mut blocks = Vec::new();
    for side in [-1.0_f32, 1.0] {
        blocks.push(slab(
            [center.x + side * half, PORTAL_LIFT, center.z],
            [0.7, PORTAL_HEIGHT + 1.4, ARCH_DEPTH],
            TRIM,
        ));
    }
    blocks.push(slab(
        [center.x, top, center.z],
        [half * 2.0 + 0.7, 0.7, ARCH_DEPTH],
        TRIM,
    ));
    blocks
}
