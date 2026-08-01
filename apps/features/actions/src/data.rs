use nightshade::prelude::Vec3;
use nightshade_gamepad::contract::{GamepadAxis, GamepadButton};
use nightshade_input::{Action, Binding, Press, Stick};
use nightshade_platform::KeyCode;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ask {
    Walk,
    Jump,
    Dash,
    NextProfile,
    Rebind,
}

impl Action for Ask {
    fn all() -> &'static [Self] {
        &[
            Self::Walk,
            Self::Jump,
            Self::Dash,
            Self::NextProfile,
            Self::Rebind,
        ]
    }

    fn name(&self) -> &'static str {
        match self {
            Self::Walk => "walk",
            Self::Jump => "jump",
            Self::Dash => "dash",
            Self::NextProfile => "next profile",
            Self::Rebind => "rebind jump",
        }
    }

    fn default_bindings(&self) -> &'static [Binding] {
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
            Self::Jump => &[
                Binding::Key(KeyCode::Space),
                Binding::Pad(GamepadButton::South),
            ],
            Self::Dash => &[
                Binding::Key(KeyCode::ShiftLeft),
                Binding::Key(KeyCode::ShiftRight),
                Binding::Pair {
                    positive: Press::Trigger(GamepadAxis::RightTrigger),
                    negative: Press::Trigger(GamepadAxis::LeftTrigger),
                },
            ],
            Self::NextProfile => &[
                Binding::Key(KeyCode::Tab),
                Binding::Pad(GamepadButton::Select),
            ],
            Self::Rebind => &[
                Binding::Key(KeyCode::KeyR),
                Binding::Pad(GamepadButton::North),
            ],
        }
    }
}

pub const SOUTHPAW: &str = "southpaw";
pub const SOUTHPAW_WALK: Binding = Binding::Keys {
    up: KeyCode::ArrowUp,
    down: KeyCode::ArrowDown,
    left: KeyCode::ArrowLeft,
    right: KeyCode::ArrowRight,
};
pub const SOUTHPAW_JUMP: Binding = Binding::Key(KeyCode::KeyF);

pub const STORE_FOLDER: &str = "nightshade";
pub const STORE_FILE: &str = "actions.toml";

pub const WALK_SPEED: f32 = 4.0;
pub const DASH_MULTIPLIER: f32 = 2.5;
pub const JUMP_SPEED: f32 = 6.0;
pub const GRAVITY: f32 = 18.0;
pub const ARENA_EXTENT: f32 = 6.0;

pub const CUBE_SCALE: Vec3 = Vec3::new(1.0, 1.0, 1.0);
pub const CUBE_ORIGIN: Vec3 = Vec3::new(0.0, 0.5, 0.0);
pub const CUBE_COLOR: [f32; 4] = [0.92, 0.56, 0.2, 1.0];
pub const FLOOR_SCALE: Vec3 = Vec3::new(14.0, 0.2, 14.0);
pub const FLOOR_ORIGIN: Vec3 = Vec3::new(0.0, -0.1, 0.0);
pub const FLOOR_COLOR: [f32; 4] = [0.28, 0.3, 0.34, 1.0];

pub const CAMERA_TARGET: Vec3 = Vec3::new(0.0, 0.5, 0.0);
pub const CAMERA_DISTANCE: f32 = 16.0;
pub const CAMERA_YAW: f32 = 0.5;
pub const CAMERA_PITCH: f32 = 0.55;

pub const PANEL_WIDTH: f32 = 360.0;
pub const PANEL_PADDING: f32 = 12.0;
pub const PANEL_SPACING: f32 = 8.0;
pub const LISTENING_HINT: &str = "press a key or a pad button for jump";
