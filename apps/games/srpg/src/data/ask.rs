use nightshade_gamepad::contract::GamepadButton;
use nightshade_input::{Action, Binding, Stick};
use nightshade_platform::KeyCode;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ask {
    Move,
    Up,
    Down,
    Left,
    Right,
    MenuUp,
    MenuDown,
    Confirm,
    Cancel,
    Begin,
    Onward,
    Pause,
    March,
    Quit,
    TurnLeft,
    TurnRight,
    ZoomIn,
    ZoomOut,
    Tilt,
    Danger,
    Bars,
}

impl Action for Ask {
    fn all() -> &'static [Self] {
        &[
            Self::Move,
            Self::Up,
            Self::Down,
            Self::Left,
            Self::Right,
            Self::MenuUp,
            Self::MenuDown,
            Self::Confirm,
            Self::Cancel,
            Self::Begin,
            Self::Onward,
            Self::Pause,
            Self::March,
            Self::Quit,
            Self::TurnLeft,
            Self::TurnRight,
            Self::ZoomIn,
            Self::ZoomOut,
            Self::Tilt,
            Self::Danger,
            Self::Bars,
        ]
    }

    fn name(&self) -> &'static str {
        match self {
            Self::Move => "move",
            Self::Up => "up",
            Self::Down => "down",
            Self::Left => "left",
            Self::Right => "right",
            Self::MenuUp => "menu up",
            Self::MenuDown => "menu down",
            Self::Confirm => "confirm",
            Self::Cancel => "cancel",
            Self::Begin => "begin",
            Self::Onward => "onward",
            Self::Pause => "pause",
            Self::March => "march",
            Self::Quit => "quit",
            Self::TurnLeft => "turn left",
            Self::TurnRight => "turn right",
            Self::ZoomIn => "zoom in",
            Self::ZoomOut => "zoom out",
            Self::Tilt => "tilt",
            Self::Danger => "danger",
            Self::Bars => "bars",
        }
    }

    fn default_bindings(&self) -> &'static [Binding] {
        match self {
            Self::Move => &[
                Binding::Keys {
                    up: KeyCode::KeyW,
                    down: KeyCode::KeyS,
                    left: KeyCode::KeyA,
                    right: KeyCode::KeyD,
                },
                Binding::Keys {
                    up: KeyCode::ArrowUp,
                    down: KeyCode::ArrowDown,
                    left: KeyCode::ArrowLeft,
                    right: KeyCode::ArrowRight,
                },
                Binding::Stick(Stick::Left),
            ],
            Self::Up => &[Binding::Pad(GamepadButton::DPadUp)],
            Self::Down => &[Binding::Pad(GamepadButton::DPadDown)],
            Self::Left => &[Binding::Pad(GamepadButton::DPadLeft)],
            Self::Right => &[Binding::Pad(GamepadButton::DPadRight)],
            Self::MenuUp => &[
                Binding::Key(KeyCode::KeyW),
                Binding::Key(KeyCode::ArrowUp),
                Binding::Pad(GamepadButton::DPadUp),
            ],
            Self::MenuDown => &[
                Binding::Key(KeyCode::KeyS),
                Binding::Key(KeyCode::ArrowDown),
                Binding::Pad(GamepadButton::DPadDown),
            ],
            Self::Confirm => &[
                Binding::Key(KeyCode::Enter),
                Binding::Key(KeyCode::Space),
                Binding::Key(KeyCode::KeyZ),
                Binding::Pad(GamepadButton::South),
            ],
            Self::Cancel => &[
                Binding::Key(KeyCode::Backspace),
                Binding::Key(KeyCode::KeyX),
                Binding::Key(KeyCode::Escape),
                Binding::Pad(GamepadButton::East),
            ],
            Self::Begin => &[
                Binding::Key(KeyCode::Enter),
                Binding::Key(KeyCode::Space),
                Binding::Pad(GamepadButton::South),
            ],
            Self::Onward => &[
                Binding::Key(KeyCode::Enter),
                Binding::Pad(GamepadButton::South),
            ],
            Self::Pause => &[
                Binding::Key(KeyCode::Escape),
                Binding::Pad(GamepadButton::Start),
            ],
            Self::March => &[Binding::Key(KeyCode::KeyM)],
            Self::Quit => &[Binding::Key(KeyCode::F4)],
            Self::TurnLeft => &[
                Binding::Key(KeyCode::KeyQ),
                Binding::Key(KeyCode::PageUp),
                Binding::Pad(GamepadButton::LeftBumper),
            ],
            Self::TurnRight => &[
                Binding::Key(KeyCode::KeyE),
                Binding::Key(KeyCode::PageDown),
                Binding::Pad(GamepadButton::RightBumper),
            ],
            Self::ZoomIn => &[
                Binding::Key(KeyCode::Equal),
                Binding::Key(KeyCode::NumpadAdd),
                Binding::Pad(GamepadButton::RightTrigger),
            ],
            Self::ZoomOut => &[
                Binding::Key(KeyCode::Minus),
                Binding::Key(KeyCode::NumpadSubtract),
                Binding::Pad(GamepadButton::LeftTrigger),
            ],
            Self::Tilt => &[
                Binding::Key(KeyCode::KeyT),
                Binding::Pad(GamepadButton::North),
            ],
            Self::Danger => &[
                Binding::Key(KeyCode::KeyV),
                Binding::Pad(GamepadButton::West),
            ],
            Self::Bars => &[
                Binding::Key(KeyCode::KeyH),
                Binding::Pad(GamepadButton::RightThumb),
            ],
        }
    }
}
