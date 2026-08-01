#[cfg(feature = "contract")]
pub mod contract;

#[cfg(feature = "contract")]
pub use contract::{Gamepad, GamepadAxis, GamepadButton};

#[cfg(all(feature = "plugin", not(target_os = "android")))]
pub use gilrs;

#[cfg(feature = "plugin")]
pub mod commands;
#[cfg(feature = "plugin")]
mod data;
#[cfg(feature = "plugin")]
mod plugin;
#[cfg(feature = "plugin")]
pub mod queries;
#[cfg(feature = "plugin")]
pub mod resources;
#[cfg(feature = "plugin")]
pub mod systems;

#[cfg(feature = "plugin")]
pub use commands::poll_gamepad;
#[cfg(feature = "plugin")]
pub use plugin::GamepadPlugin;
#[cfg(feature = "plugin")]
pub use queries::query_active_gamepad;
#[cfg(feature = "plugin")]
pub use resources::GamepadBackend;
