pub mod android;
pub mod contract;

pub mod commands;
#[cfg(feature = "plugin")]
pub mod plugin;
pub mod queries;
#[cfg(feature = "plugin")]
pub mod systems;

pub use winit;

#[cfg(feature = "plugin")]
pub use crate::plugin::{PlatformPlugin, install, register_frame_systems};
pub use winit::event::{ElementState, WindowEvent};
pub use winit::event_loop::ActiveEventLoop;
pub use winit::keyboard::KeyCode;
pub use winit::window::CursorIcon;
pub use winit::window::Window as WinitWindow;

pub mod prelude {
    pub use crate::contract::AppEvent;
    pub use crate::contract::DroppedFile;
    pub use crate::contract::input::Input;
    pub use crate::contract::input::input_mode::InputMode;
    pub use crate::contract::input::mouse::MouseState;
    pub use crate::contract::input::touch::{TouchGesture, TouchPhase};
    pub use crate::contract::input_capture::InputCapture;
    pub use crate::contract::monitor::{Monitor, Monitors};
    pub use crate::contract::time::Time;
    pub use crate::contract::window::Window;
    pub use winit;

    pub use crate::commands::open_url;
    pub use crate::commands::time::{pause, set_paused, set_time_scale, unpause};
    pub use crate::commands::window::{set_cursor_locked, set_cursor_visible};
    #[cfg(feature = "plugin")]
    pub use crate::plugin::{PlatformPlugin, install, register_frame_systems};
    pub use crate::queries::input_capture::{keyboard_captured, pointer_captured};
    pub use crate::queries::time::{is_paused, time_scale};
    pub use crate::queries::window::{window_scale_factor, window_viewport_size};
    #[cfg(feature = "plugin")]
    pub use crate::systems::{apply_ime_allowed_system, escape_key_exit_system, set_ime_allowed};
    pub use winit::event::{ElementState, ElementState as KeyState, MouseButton, WindowEvent};
    pub use winit::keyboard::KeyCode;
    pub use winit::window;
}
