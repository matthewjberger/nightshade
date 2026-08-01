use nightshade_gamepad::contract::{GamepadAxis, GamepadButton};
use nightshade_input::{Action, Binding, Press, Stick};
use nightshade_platform::KeyCode;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ask {
    Walk,
    Sprint,
    Use,
    Fly,
    Steer,
    Drive,
    Handbrake,
    Rise,
    Look,
    Menu,
}

impl Action for Ask {
    fn all() -> &'static [Self] {
        &[
            Self::Walk,
            Self::Sprint,
            Self::Use,
            Self::Fly,
            Self::Steer,
            Self::Drive,
            Self::Handbrake,
            Self::Rise,
            Self::Look,
            Self::Menu,
        ]
    }

    fn name(&self) -> &'static str {
        match self {
            Self::Walk => "walk",
            Self::Sprint => "sprint",
            Self::Use => "use",
            Self::Fly => "fly",
            Self::Steer => "steer",
            Self::Drive => "drive",
            Self::Handbrake => "handbrake",
            Self::Rise => "rise",
            Self::Look => "look",
            Self::Menu => "menu",
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
                Binding::Keys {
                    up: KeyCode::ArrowUp,
                    down: KeyCode::ArrowDown,
                    left: KeyCode::ArrowLeft,
                    right: KeyCode::ArrowRight,
                },
                Binding::Stick(Stick::Left),
            ],
            Self::Sprint => &[
                Binding::Key(KeyCode::ShiftLeft),
                Binding::Key(KeyCode::ShiftRight),
                Binding::Pad(GamepadButton::West),
                Binding::Pad(GamepadButton::LeftThumb),
            ],
            Self::Use => &[
                Binding::Key(KeyCode::KeyE),
                Binding::Pad(GamepadButton::North),
            ],
            Self::Fly => &[
                Binding::Key(KeyCode::KeyF),
                Binding::Pad(GamepadButton::Select),
            ],
            Self::Steer => &[
                Binding::Pair {
                    positive: Press::Key(KeyCode::KeyD),
                    negative: Press::Key(KeyCode::KeyA),
                },
                Binding::Pair {
                    positive: Press::Key(KeyCode::ArrowRight),
                    negative: Press::Key(KeyCode::ArrowLeft),
                },
                Binding::Stick(Stick::Left),
            ],
            Self::Drive => &[
                Binding::Pair {
                    positive: Press::Key(KeyCode::KeyW),
                    negative: Press::Key(KeyCode::KeyS),
                },
                Binding::Pair {
                    positive: Press::Key(KeyCode::ArrowUp),
                    negative: Press::Key(KeyCode::ArrowDown),
                },
                Binding::Pair {
                    positive: Press::Trigger(GamepadAxis::RightTrigger),
                    negative: Press::Trigger(GamepadAxis::LeftTrigger),
                },
                Binding::Pair {
                    positive: Press::Pad(GamepadButton::South),
                    negative: Press::Pad(GamepadButton::West),
                },
            ],
            Self::Handbrake => &[
                Binding::Key(KeyCode::Space),
                Binding::Pad(GamepadButton::East),
            ],
            Self::Rise => &[
                Binding::Pair {
                    positive: Press::Key(KeyCode::KeyE),
                    negative: Press::Key(KeyCode::KeyQ),
                },
                Binding::Pair {
                    positive: Press::Key(KeyCode::Space),
                    negative: Press::Key(KeyCode::ControlLeft),
                },
                Binding::Pair {
                    positive: Press::Trigger(GamepadAxis::RightTrigger),
                    negative: Press::Trigger(GamepadAxis::LeftTrigger),
                },
            ],
            Self::Look => &[Binding::Stick(Stick::Right)],
            Self::Menu => &[
                Binding::Key(KeyCode::Escape),
                Binding::Pad(GamepadButton::Start),
            ],
        }
    }
}
