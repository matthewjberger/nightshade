pub mod input;
pub mod input_capture;
pub mod monitor;
pub mod time;
pub mod window;

pub use input::*;
pub use input_capture::*;
pub use time::*;
pub use window::*;

#[derive(Clone)]
pub enum AppEvent {
    Keyboard {
        key: winit::keyboard::KeyCode,
        state: winit::event::ElementState,
    },
    Mouse {
        button: winit::event::MouseButton,
        state: winit::event::ElementState,
    },
    FileDropped(DroppedFile),
    FileDroppedPath(std::path::PathBuf),
    FileHovered(std::path::PathBuf),
    FileHoverCancelled,
    Suspended,
    Resumed,
}

#[derive(Clone)]
pub struct DroppedFile {
    pub name: String,
    pub data: Vec<u8>,
}
