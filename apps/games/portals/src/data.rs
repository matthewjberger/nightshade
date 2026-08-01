pub const TITLE: &str = "Portals";

pub const ROOM_HEIGHT: f32 = 4.5;
pub const WALL_THICKNESS: f32 = 0.6;
pub const HALF_SPAN: f32 = 12.0;
pub const HALF_DEPTH: f32 = 8.0;
pub const DOORWAY_HALF: f32 = 1.6;

pub const START: [f32; 3] = [-6.0, 0.0, 3.0];
pub const START_YAW: f32 = 0.0;
pub const EYE_HEIGHT: f32 = 1.65;
pub const PLAYER_HALF_WIDTH: f32 = 0.35;
pub const PLAYER_HEIGHT: f32 = 1.8;
pub const WALK_SPEED: f32 = 4.5;
pub const GRAVITY: f32 = 20.0;
pub const TERMINAL_FALL: f32 = 42.0;
pub const STEP_LIMIT: f32 = 0.2;
pub const MAX_SLICES: u32 = 24;
pub const MOUSE_SENSITIVITY: f32 = 0.0022;
pub const PITCH_LIMIT: f32 = 1.5;
pub const STICK_LOOK_SPEED: f32 = 2.6;
pub const TRAVEL_MARGIN: f32 = 0.18;
pub const TRAVEL_HOLD: f32 = 0.28;
pub const TRAVEL_CLEARANCE: f32 = 0.32;
pub const TRAVEL_STEPS: u32 = 14;
pub const TRAVEL_STEP: f32 = 0.18;
pub const TRAVEL_HEADING: f32 = 0.2;

pub const AVATAR_LAYER: u32 = 2;
pub const AVATAR_RADIUS: f32 = 0.35;
pub const AVATAR_COLOR: [f32; 4] = [0.84, 0.86, 0.92, 1.0];
pub const LABEL_LIFT: f32 = 0.55;
pub const LABEL_SIZE: f32 = 56.0;
pub const LABEL_SCALE: f32 = 1.0;
pub const LABEL_OUTLINE: f32 = 3.0;

pub const FIELD_OF_VIEW: f32 = 1.2;
pub const NEAR_PLANE: f32 = 0.05;
pub const FAR_PLANE: f32 = 400.0;
pub const PLANE_BIAS: f32 = 0.05;
pub const REACH: f32 = 90.0;

pub const PORTAL_WIDTH: f32 = 1.5;
pub const PORTAL_HEIGHT: f32 = 2.6;
pub const PORTAL_LIFT: f32 = 1.45;
pub const PORTAL_SURFACE_OFFSET: f32 = 0.06;
pub const PORTAL_DEPTH: u32 = 3;
pub const PORTAL_MAX_DISTANCE: f32 = 0.0;
pub const PORTAL_TARGET_MAX: u32 = 1400;

pub const BLUE_RIM: [f32; 4] = [0.22, 0.62, 1.0, 1.0];
pub const ORANGE_RIM: [f32; 4] = [1.0, 0.50, 0.10, 1.0];

pub const SPARKS_PER_RIM: usize = 14;
pub const SPARK_RATE: f32 = 30.0;
pub const SPARK_LIFE: [f32; 2] = [0.18, 0.52];
pub const SPARK_SPEED: [f32; 2] = [0.4, 1.9];
pub const SPARK_SPREAD: f32 = 0.85;
pub const SPARK_SIZE: [f32; 2] = [0.13, 0.0];
pub const SPARK_GRAVITY: f32 = -2.4;
pub const SPARK_DRAG: f32 = 1.4;
pub const SPARK_GLOW: f32 = 1.7;
pub const SPARK_TURNS: f32 = 0.16;
pub const SPARK_STAND_OFF: f32 = 0.06;
pub const SPARK_OUTWARD: f32 = 0.42;

pub const MENU_WIDTH: f32 = 340.0;
pub const MENU_HEIGHT: f32 = 198.0;

const COOL_WALL: [f32; 4] = [0.29, 0.42, 0.56, 1.0];
const COOL_FLOOR: [f32; 4] = [0.18, 0.25, 0.33, 1.0];
const WARM_WALL: [f32; 4] = [0.62, 0.38, 0.25, 1.0];
const WARM_FLOOR: [f32; 4] = [0.36, 0.23, 0.16, 1.0];

pub struct Slab {
    pub center: [f32; 3],
    pub size: [f32; 3],
    pub color: [f32; 4],
    pub roughness: f32,
}

pub struct Hit {
    pub point: nightshade::prelude::Vec3,
    pub normal: nightshade::prelude::Vec3,
    pub distance: f32,
}

fn slab(center: [f32; 3], size: [f32; 3], color: [f32; 4]) -> Slab {
    Slab {
        center,
        size,
        color,
        roughness: 0.85,
    }
}

pub fn rooms() -> Vec<Slab> {
    let span = HALF_SPAN;
    let depth = HALF_DEPTH;
    let height = ROOM_HEIGHT;
    let thickness = WALL_THICKNESS;
    let middle = span * 0.5;
    let gate_span = depth - DOORWAY_HALF;
    let gate = DOORWAY_HALF + gate_span * 0.5;
    vec![
        slab(
            [-middle, -thickness * 0.5, 0.0],
            [span, thickness, depth * 2.0],
            COOL_FLOOR,
        ),
        slab(
            [middle, -thickness * 0.5, 0.0],
            [span, thickness, depth * 2.0],
            WARM_FLOOR,
        ),
        slab(
            [-span - thickness * 0.5, height * 0.5, 0.0],
            [thickness, height, depth * 2.0],
            COOL_WALL,
        ),
        slab(
            [span + thickness * 0.5, height * 0.5, 0.0],
            [thickness, height, depth * 2.0],
            WARM_WALL,
        ),
        slab(
            [-middle, height * 0.5, -depth - thickness * 0.5],
            [span, height, thickness],
            COOL_WALL,
        ),
        slab(
            [-middle, height * 0.5, depth + thickness * 0.5],
            [span, height, thickness],
            COOL_WALL,
        ),
        slab(
            [middle, height * 0.5, -depth - thickness * 0.5],
            [span, height, thickness],
            WARM_WALL,
        ),
        slab(
            [middle, height * 0.5, depth + thickness * 0.5],
            [span, height, thickness],
            WARM_WALL,
        ),
        slab(
            [0.0, height * 0.5, -gate],
            [thickness, height, gate_span],
            COOL_WALL,
        ),
        slab(
            [0.0, height * 0.5, gate],
            [thickness, height, gate_span],
            COOL_WALL,
        ),
    ]
}

pub fn props() -> Vec<Slab> {
    vec![
        slab([-9.5, 1.4, -4.0], [1.0, 2.8, 1.0], [0.88, 0.92, 0.96, 1.0]),
        slab([-4.0, 0.6, -5.5], [1.8, 1.2, 1.8], [0.22, 0.72, 0.76, 1.0]),
        slab([-8.0, 0.4, 4.5], [2.6, 0.8, 2.6], [0.38, 0.46, 0.84, 1.0]),
        slab([9.5, 1.4, -4.0], [1.0, 2.8, 1.0], [0.95, 0.83, 0.28, 1.0]),
        slab([4.0, 0.6, -5.5], [1.8, 1.2, 1.8], [0.86, 0.25, 0.18, 1.0]),
        slab([8.0, 0.4, 4.5], [2.6, 0.8, 2.6], [0.92, 0.51, 0.16, 1.0]),
    ]
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ask {
    Walk,
    Look,
    FireBlue,
    FireOrange,
    Pause,
}

impl nightshade_input::Action for Ask {
    fn all() -> &'static [Self] {
        &[
            Self::Walk,
            Self::Look,
            Self::FireBlue,
            Self::FireOrange,
            Self::Pause,
        ]
    }

    fn name(&self) -> &'static str {
        match self {
            Self::Walk => "walk",
            Self::Look => "look",
            Self::FireBlue => "fire blue",
            Self::FireOrange => "fire orange",
            Self::Pause => "pause",
        }
    }

    fn default_bindings(&self) -> &'static [nightshade_input::Binding] {
        use nightshade_gamepad::contract::GamepadButton;
        use nightshade_input::{Binding, Pointer, Stick};
        use nightshade_platform::KeyCode;
        match self {
            Self::Walk => &[
                Binding::Keys {
                    up: KeyCode::KeyW,
                    down: KeyCode::KeyS,
                    left: KeyCode::KeyA,
                    right: KeyCode::KeyD,
                },
                Binding::Stick(Stick::Left),
            ],
            Self::Look => &[Binding::Stick(Stick::Right)],
            Self::FireBlue => &[
                Binding::Pointer(Pointer::Left),
                Binding::Pad(GamepadButton::RightTrigger),
                Binding::Pad(GamepadButton::RightBumper),
            ],
            Self::FireOrange => &[
                Binding::Pointer(Pointer::Right),
                Binding::Pad(GamepadButton::LeftTrigger),
                Binding::Pad(GamepadButton::LeftBumper),
            ],
            Self::Pause => &[
                Binding::Key(KeyCode::Escape),
                Binding::Pad(GamepadButton::Start),
            ],
        }
    }
}
