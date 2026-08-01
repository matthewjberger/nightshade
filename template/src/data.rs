use nightshade_gamepad::contract::GamepadButton;
use nightshade_input::{Action, Binding};
use nightshade_platform::KeyCode;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ask {
    Ping,
    Pause,
    Quit,
}

impl Action for Ask {
    fn all() -> &'static [Self] {
        &[Self::Ping, Self::Pause, Self::Quit]
    }

    fn name(&self) -> &'static str {
        match self {
            Self::Ping => "ping",
            Self::Pause => "pause",
            Self::Quit => "quit",
        }
    }

    fn default_bindings(&self) -> &'static [Binding] {
        match self {
            Self::Ping => &[
                Binding::Key(KeyCode::KeyP),
                Binding::Pad(GamepadButton::South),
            ],
            Self::Pause => &[
                Binding::Key(KeyCode::Escape),
                Binding::Pad(GamepadButton::Start),
            ],
            Self::Quit => &[Binding::Key(KeyCode::KeyQ)],
        }
    }
}
