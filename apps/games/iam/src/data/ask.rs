use nightshade_input::{Action, Binding};
use nightshade_platform::KeyCode;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ask {
    Erase,
    Submit,
    Clear,
}

impl Action for Ask {
    fn all() -> &'static [Self] {
        &[Self::Erase, Self::Submit, Self::Clear]
    }

    fn name(&self) -> &'static str {
        match self {
            Self::Erase => "erase",
            Self::Submit => "submit",
            Self::Clear => "clear",
        }
    }

    fn default_bindings(&self) -> &'static [Binding] {
        match self {
            Self::Erase => &[Binding::Key(KeyCode::Backspace)],
            Self::Submit => &[Binding::Key(KeyCode::Enter)],
            Self::Clear => &[Binding::Key(KeyCode::Escape)],
        }
    }
}
