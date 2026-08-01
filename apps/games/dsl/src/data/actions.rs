use nightshade_gamepad::contract::GamepadButton;
use nightshade_input::{Action, Binding, Press, Stick};
use nightshade_platform::KeyCode;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ask {
    Move,
    StepAcross,
    StepAlong,
    Serve,
}

impl Action for Ask {
    fn all() -> &'static [Self] {
        &[Self::Move, Self::StepAcross, Self::StepAlong, Self::Serve]
    }

    fn name(&self) -> &'static str {
        match self {
            Self::Move => "move",
            Self::StepAcross => "step across",
            Self::StepAlong => "step along",
            Self::Serve => "serve",
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
            Self::StepAcross => &[Binding::Pair {
                positive: Press::Pad(GamepadButton::DPadRight),
                negative: Press::Pad(GamepadButton::DPadLeft),
            }],
            Self::StepAlong => &[Binding::Pair {
                positive: Press::Pad(GamepadButton::DPadUp),
                negative: Press::Pad(GamepadButton::DPadDown),
            }],
            Self::Serve => &[
                Binding::Key(KeyCode::Space),
                Binding::Pad(GamepadButton::South),
            ],
        }
    }
}
