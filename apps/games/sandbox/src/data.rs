use nightshade_gamepad::contract::GamepadButton;
use nightshade_input::{Action, Binding, Stick};
use nightshade_platform::KeyCode;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ask {
    Walk,
    Jump,
    Crouch,
    Flip,
    Reset,
    Pause,
    Quit,
}

impl Action for Ask {
    fn all() -> &'static [Self] {
        &[
            Self::Walk,
            Self::Jump,
            Self::Crouch,
            Self::Flip,
            Self::Reset,
            Self::Pause,
            Self::Quit,
        ]
    }

    fn name(&self) -> &'static str {
        match self {
            Self::Walk => "walk",
            Self::Jump => "jump",
            Self::Crouch => "crouch",
            Self::Flip => "flip",
            Self::Reset => "reset",
            Self::Pause => "pause",
            Self::Quit => "quit",
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
            Self::Crouch => &[
                Binding::Key(KeyCode::ControlLeft),
                Binding::Key(KeyCode::KeyC),
                Binding::Pad(GamepadButton::East),
            ],
            Self::Flip => &[
                Binding::Key(KeyCode::KeyF),
                Binding::Pad(GamepadButton::West),
            ],
            Self::Reset => &[Binding::Key(KeyCode::KeyR)],
            Self::Pause => &[
                Binding::Key(KeyCode::Escape),
                Binding::Pad(GamepadButton::Start),
            ],
            Self::Quit => &[Binding::Key(KeyCode::Escape)],
        }
    }
}
