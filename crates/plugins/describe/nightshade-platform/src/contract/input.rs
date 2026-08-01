pub mod android;
pub mod input_mode;
pub mod keyboard;
pub mod mouse;
pub mod touch;

pub use android::*;
pub use input_mode::*;
pub use keyboard::*;
pub use mouse::*;
pub use touch::*;

#[derive(Default)]
pub struct Input {
    pub keyboard: Keyboard,
    pub mouse: Mouse,
    pub touch: Touch,
    pub input_mode: InputMode,
    pub events: Vec<crate::contract::AppEvent>,
}
