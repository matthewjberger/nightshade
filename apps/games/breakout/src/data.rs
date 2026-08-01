pub const TITLE: &str = "Breakout";

pub const FIELD_HALF_WIDTH: f32 = 16.0;
pub const FIELD_HALF_HEIGHT: f32 = 12.0;
pub const WALL_HALF_THICKNESS: f32 = 0.5;
pub const FLOOR_Y: f32 = -FIELD_HALF_HEIGHT - 1.0;

pub const INNER_HALF_WIDTH: f32 = FIELD_HALF_WIDTH - WALL_HALF_THICKNESS;
pub const INNER_TOP: f32 = FIELD_HALF_HEIGHT - WALL_HALF_THICKNESS;

pub const PADDLE_HALF_WIDTH: f32 = 2.5;
pub const PADDLE_HALF_HEIGHT: f32 = 0.4;
pub const PADDLE_Y: f32 = -9.5;
pub const PADDLE_SPEED: f32 = 26.0;

pub const BALL_RADIUS: f32 = 0.35;
pub const BALL_SPEED: f32 = 20.0;
pub const BALL_STEP: f32 = 0.15;
pub const LONGEST_FRAME: f32 = 1.0 / 30.0;
pub const MAX_BOUNCE_RADIANS: f32 = std::f32::consts::FRAC_PI_3;
pub const SERVE_HEIGHT: f32 = PADDLE_HALF_HEIGHT + BALL_RADIUS + 0.1;
pub const SERVE_SPREAD: f32 = 0.6;

pub const WALLS: [([f32; 2], [f32; 2]); 3] = [
    (
        [-FIELD_HALF_WIDTH, 0.0],
        [WALL_HALF_THICKNESS, FIELD_HALF_HEIGHT + WALL_HALF_THICKNESS],
    ),
    (
        [FIELD_HALF_WIDTH, 0.0],
        [WALL_HALF_THICKNESS, FIELD_HALF_HEIGHT + WALL_HALF_THICKNESS],
    ),
    (
        [0.0, FIELD_HALF_HEIGHT],
        [FIELD_HALF_WIDTH + WALL_HALF_THICKNESS, WALL_HALF_THICKNESS],
    ),
];

pub const BRICK_COLUMNS: u32 = 8;
pub const BRICK_ROWS: u32 = 5;
pub const BRICK_HALF_WIDTH: f32 = 1.6;
pub const BRICK_HALF_HEIGHT: f32 = 0.55;
pub const BRICK_SPACING_X: f32 = 3.6;
pub const BRICK_SPACING_Y: f32 = 1.6;
pub const BRICK_TOP_Y: f32 = 9.5;

pub const STARTING_LIVES: u32 = 3;

pub const CLEAR_COLOR: [f32; 4] = [0.006, 0.007, 0.013, 1.0];
pub const WALL_COLOR: [f32; 4] = [0.147, 0.179, 0.296, 1.0];
pub const PADDLE_COLOR: [f32; 4] = [0.890, 0.911, 1.000, 1.0];
pub const BALL_COLOR: [f32; 4] = [1.000, 0.749, 0.100, 1.0];

pub const ROW_COLORS: [[f32; 4]; BRICK_ROWS as usize] = [
    [0.808, 0.040, 0.068, 1.0],
    [0.911, 0.233, 0.022, 1.0],
    [0.955, 0.638, 0.033, 1.0],
    [0.051, 0.571, 0.119, 1.0],
    [0.051, 0.263, 0.890, 1.0],
];

pub const ROW_POINTS: [u32; BRICK_ROWS as usize] = [50, 40, 30, 20, 10];

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ask {
    Steer,
    Serve,
}

impl nightshade_input::Action for Ask {
    fn all() -> &'static [Self] {
        &[Self::Steer, Self::Serve]
    }

    fn name(&self) -> &'static str {
        match self {
            Self::Steer => "steer",
            Self::Serve => "serve",
        }
    }

    fn default_bindings(&self) -> &'static [nightshade_input::Binding] {
        use nightshade_gamepad::contract::GamepadButton;
        use nightshade_input::{Binding, Press, Stick};
        use nightshade_platform::KeyCode;
        match self {
            Self::Steer => &[
                Binding::Pair {
                    positive: Press::Key(KeyCode::KeyD),
                    negative: Press::Key(KeyCode::KeyA),
                },
                Binding::Pair {
                    positive: Press::Key(KeyCode::ArrowRight),
                    negative: Press::Key(KeyCode::ArrowLeft),
                },
                Binding::Pair {
                    positive: Press::Pad(GamepadButton::DPadRight),
                    negative: Press::Pad(GamepadButton::DPadLeft),
                },
                Binding::Stick(Stick::Left),
            ],
            Self::Serve => &[
                Binding::Key(KeyCode::Space),
                Binding::Pad(GamepadButton::South),
            ],
        }
    }
}
