use nightshade_gamepad::contract::{GamepadAxis, GamepadButton};
use nightshade_input::{Action, Binding, Pointer, Press, Stick};
use nightshade_platform::KeyCode;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ask {
    Take,
    Strike,
    Torch,
    Pause,
    Confirm,
    OpenPockets,
    ClosePockets,
    OpenMap,
    CloseMap,
    PickPipe,
    PickHandgun,
    Reach,
    Throw,
    PushHold,
    CloseNote,
    NextLine,
    NavigateUp,
    NavigateDown,
    NavigateLeft,
    NavigateRight,
    Spin,
}

impl Action for Ask {
    fn all() -> &'static [Self] {
        &[
            Self::Take,
            Self::Strike,
            Self::Torch,
            Self::Pause,
            Self::Confirm,
            Self::OpenPockets,
            Self::ClosePockets,
            Self::OpenMap,
            Self::CloseMap,
            Self::PickPipe,
            Self::PickHandgun,
            Self::Reach,
            Self::Throw,
            Self::PushHold,
            Self::CloseNote,
            Self::NextLine,
            Self::NavigateUp,
            Self::NavigateDown,
            Self::NavigateLeft,
            Self::NavigateRight,
            Self::Spin,
        ]
    }

    fn name(&self) -> &'static str {
        match self {
            Self::Take => "take",
            Self::Strike => "strike",
            Self::Torch => "torch",
            Self::Pause => "pause",
            Self::Confirm => "confirm",
            Self::OpenPockets => "open pockets",
            Self::ClosePockets => "close pockets",
            Self::OpenMap => "open map",
            Self::CloseMap => "close map",
            Self::PickPipe => "pick pipe",
            Self::PickHandgun => "pick handgun",
            Self::Reach => "reach",
            Self::Throw => "throw",
            Self::PushHold => "push hold",
            Self::CloseNote => "close note",
            Self::NextLine => "next line",
            Self::NavigateUp => "navigate up",
            Self::NavigateDown => "navigate down",
            Self::NavigateLeft => "navigate left",
            Self::NavigateRight => "navigate right",
            Self::Spin => "spin",
        }
    }

    fn default_bindings(&self) -> &'static [Binding] {
        match self {
            Self::Take => &[
                Binding::Key(KeyCode::KeyE),
                Binding::Pad(GamepadButton::South),
            ],
            Self::Strike => &[
                Binding::Pointer(Pointer::Left),
                Binding::Pad(GamepadButton::West),
            ],
            Self::Torch => &[
                Binding::Key(KeyCode::KeyF),
                Binding::Pad(GamepadButton::North),
            ],
            Self::Pause => &[
                Binding::Key(KeyCode::KeyP),
                Binding::Key(KeyCode::Escape),
                Binding::Pad(GamepadButton::Start),
                Binding::Pad(GamepadButton::East),
            ],
            Self::Confirm => &[
                Binding::Key(KeyCode::Enter),
                Binding::Key(KeyCode::Space),
                Binding::Pad(GamepadButton::South),
            ],
            Self::OpenPockets => &[
                Binding::Key(KeyCode::KeyI),
                Binding::Pad(GamepadButton::DPadDown),
            ],
            Self::ClosePockets => &[Binding::Key(KeyCode::KeyI)],
            Self::OpenMap => &[
                Binding::Key(KeyCode::KeyM),
                Binding::Pad(GamepadButton::DPadUp),
            ],
            Self::CloseMap => &[
                Binding::Key(KeyCode::KeyM),
                Binding::Pad(GamepadButton::DPadUp),
            ],
            Self::PickPipe => &[
                Binding::Key(KeyCode::Digit1),
                Binding::Pad(GamepadButton::DPadLeft),
            ],
            Self::PickHandgun => &[
                Binding::Key(KeyCode::Digit2),
                Binding::Pad(GamepadButton::DPadRight),
            ],
            Self::Reach => &[
                Binding::Pointer(Pointer::Right),
                Binding::Pad(GamepadButton::LeftBumper),
            ],
            Self::Throw => &[
                Binding::Pointer(Pointer::Left),
                Binding::Pad(GamepadButton::West),
                Binding::Pad(GamepadButton::RightBumper),
            ],
            Self::PushHold => &[Binding::Pair {
                positive: Press::Trigger(GamepadAxis::RightTrigger),
                negative: Press::Trigger(GamepadAxis::LeftTrigger),
            }],
            Self::CloseNote => &[Binding::Key(KeyCode::KeyE)],
            Self::NextLine => &[Binding::Key(KeyCode::KeyE)],
            Self::NavigateUp => &[
                Binding::Key(KeyCode::ArrowUp),
                Binding::Key(KeyCode::KeyW),
                Binding::Pad(GamepadButton::DPadUp),
            ],
            Self::NavigateDown => &[
                Binding::Key(KeyCode::ArrowDown),
                Binding::Key(KeyCode::KeyS),
                Binding::Pad(GamepadButton::DPadDown),
            ],
            Self::NavigateLeft => &[
                Binding::Key(KeyCode::ArrowLeft),
                Binding::Key(KeyCode::KeyA),
                Binding::Pad(GamepadButton::DPadLeft),
            ],
            Self::NavigateRight => &[
                Binding::Key(KeyCode::ArrowRight),
                Binding::Key(KeyCode::KeyD),
                Binding::Pad(GamepadButton::DPadRight),
            ],
            Self::Spin => &[
                Binding::Keys {
                    up: KeyCode::ArrowUp,
                    down: KeyCode::ArrowDown,
                    left: KeyCode::ArrowLeft,
                    right: KeyCode::ArrowRight,
                },
                Binding::Stick(Stick::Right),
            ],
        }
    }
}
