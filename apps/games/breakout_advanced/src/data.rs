pub const TITLE: &str = "Breakout Advanced";

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
pub const MAX_BOUNCE_RADIANS: f32 = std::f32::consts::FRAC_PI_3;
pub const SERVE_HEIGHT: f32 = PADDLE_HALF_HEIGHT + BALL_RADIUS + 0.1;

pub const BRICK_COLUMNS: u32 = 8;
pub const BRICK_ROWS: u32 = 6;
pub const BRICK_HALF_WIDTH: f32 = 1.6;
pub const BRICK_HALF_HEIGHT: f32 = 0.55;
pub const BRICK_SPACING_X: f32 = 3.6;
pub const BRICK_SPACING_Y: f32 = 1.6;
pub const BRICK_TOP_Y: f32 = 9.5;

pub const STARTING_LIVES: u32 = 3;

pub const LEVEL_COUNT: usize = 5;

pub const LEVELS: [&[&str]; LEVEL_COUNT] = [
    &["11111111", "11111111", "11111111"],
    &["22222222", "11111111", "1.1.1.1.", ".1.1.1.1"],
    &["..3333..", ".322223.", "32111123", "11111111"],
    &["31.11.13", "21.11.12", "11.11.11", "11111111"],
    &[
        "33333333", "22222222", "22222222", "11111111", "11111111", "11111111",
    ],
];

pub const LEVEL_PAR: [f32; LEVEL_COUNT] = [35.0, 45.0, 60.0, 55.0, 90.0];
pub const PAR_BONUS: u32 = 500;

pub const TIER_SHADE: [f32; 3] = [1.0, 0.55, 0.30];

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
    [0.342, 0.073, 0.828, 1.0],
];

pub const ROW_POINTS: [u32; BRICK_ROWS as usize] = [60, 50, 40, 30, 20, 10];

pub const HUD_WIDTH: f32 = 1180.0;
pub const HUD_HEIGHT: f32 = 64.0;
pub const HUD_MARGIN: f32 = 18.0;
pub const HUD_TEXT_SIZE: f32 = 24.0;
pub const HUD_TEXT_COLOR: [f32; 4] = [0.92, 0.94, 1.0, 1.0];

pub const PANEL_BACKGROUND: [f32; 4] = [0.03, 0.035, 0.06, 0.92];
pub const PANEL_BORDER: [f32; 4] = [1.0, 1.0, 1.0, 0.14];
pub const PANEL_TEXT: [f32; 4] = [0.74, 0.78, 0.88, 1.0];
pub const TITLE_WIDTH: f32 = 480.0;
pub const TITLE_HEIGHT: f32 = 282.0;
pub const MENU_WIDTH: f32 = 340.0;
pub const MENU_HEIGHT: f32 = 250.0;
pub const OVER_WIDTH: f32 = 400.0;
pub const OVER_HEIGHT: f32 = 230.0;

pub const LEAN_STIFFNESS: f32 = 220.0;
pub const LEAN_DAMPING: f32 = 18.0;
pub const LEAN_GAIN: f32 = 0.020;
pub const LEAN_LIMIT: f32 = 0.45;
pub const PADDLE_SQUASH_GAIN: f32 = 0.55;

pub const STRETCH_GAIN: f32 = 0.30;
pub const SQUASH_GAIN: f32 = 0.55;
pub const SQUASH_DECAY: f32 = 7.0;

pub const TRAIL_STEP: f32 = 0.55;
pub const TRAIL_LIFE: f32 = 0.28;
pub const TRAIL_SCALE: f32 = 0.85;

pub const POP_SECONDS: f32 = 0.22;
pub const POP_SWELL: f32 = 0.85;
pub const GROW_SECONDS: f32 = 0.26;
pub const GROW_STAGGER: f32 = 0.022;

pub const SHAKE_WALL: f32 = 0.16;
pub const SHAKE_PADDLE: f32 = 0.24;
pub const SHAKE_CHIP: f32 = 0.30;
pub const SHAKE_BRICK: f32 = 0.55;
pub const SHAKE_LOST: f32 = 1.30;
pub const SHAKE_LIMIT: f32 = 1.60;
pub const SHAKE_DECAY: f32 = 6.5;
pub const SHAKE_REACH: f32 = 0.55;

pub const TINT_DRIFT: f32 = 2.4;
pub const TINT_STRENGTH: f32 = 0.085;

pub const ZOOM_NEAR: f32 = 0.86;
pub const ZOOM_EASE: f32 = 2.0;
pub const ZOOM_FROM: u32 = 6;

pub const SLOW_SECONDS: f32 = 0.55;
pub const SLOW_SCALE: f32 = 0.22;
pub const SLOW_EASE: f32 = 5.0;

pub const DEBRIS_COUNT: u32 = 26;
pub const DEBRIS_LIFE: (f32, f32) = (0.28, 0.62);
pub const DEBRIS_SPEED: (f32, f32) = (4.0, 13.0);
pub const DEBRIS_SIZE: (f32, f32) = (0.34, 0.0);
pub const DEBRIS_GRAVITY: f32 = -16.0;
pub const SPRITE_TEXELS: u32 = 128;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ask {
    Steer,
    Serve,
    Pause,
    Confirm,
    Back,
    MenuUp,
    MenuDown,
    Browse,
}

impl nightshade_input::Action for Ask {
    fn all() -> &'static [Self] {
        &[
            Self::Steer,
            Self::Serve,
            Self::Pause,
            Self::Confirm,
            Self::Back,
            Self::MenuUp,
            Self::MenuDown,
            Self::Browse,
        ]
    }

    fn name(&self) -> &'static str {
        match self {
            Self::Steer => "steer",
            Self::Serve => "serve",
            Self::Pause => "pause",
            Self::Confirm => "confirm",
            Self::Back => "back",
            Self::MenuUp => "menu up",
            Self::MenuDown => "menu down",
            Self::Browse => "browse",
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
            Self::Pause => &[
                Binding::Key(KeyCode::Escape),
                Binding::Pad(GamepadButton::Start),
            ],
            Self::Confirm => &[Binding::Pad(GamepadButton::South)],
            Self::Back => &[Binding::Pad(GamepadButton::East)],
            Self::MenuUp => &[Binding::Pad(GamepadButton::DPadUp)],
            Self::MenuDown => &[Binding::Pad(GamepadButton::DPadDown)],
            Self::Browse => &[Binding::Stick(Stick::Left)],
        }
    }
}
