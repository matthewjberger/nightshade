use nightshade_gamepad::contract::GamepadButton;
use nightshade_input::{Action, Binding};
use nightshade_platform::KeyCode;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ask {
    ToggleWalk,
}

impl Action for Ask {
    fn all() -> &'static [Self] {
        &[Self::ToggleWalk]
    }

    fn name(&self) -> &'static str {
        match self {
            Self::ToggleWalk => "toggle walk",
        }
    }

    fn default_bindings(&self) -> &'static [Binding] {
        match self {
            Self::ToggleWalk => &[
                Binding::Key(KeyCode::Tab),
                Binding::Pad(GamepadButton::Start),
            ],
        }
    }
}
