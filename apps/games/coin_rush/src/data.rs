use nightshade_gamepad::contract::GamepadButton;
use nightshade_input::{Action, Binding};
use nightshade_platform::KeyCode;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ask {
    Pause,
}

impl Action for Ask {
    fn all() -> &'static [Self] {
        &[Self::Pause]
    }

    fn name(&self) -> &'static str {
        match self {
            Self::Pause => "pause",
        }
    }

    fn default_bindings(&self) -> &'static [Binding] {
        match self {
            Self::Pause => &[
                Binding::Key(KeyCode::KeyP),
                Binding::Pad(GamepadButton::Start),
            ],
        }
    }
}
