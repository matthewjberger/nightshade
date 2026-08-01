pub mod components;
pub mod prelude;
#[cfg(feature = "plugin")]
pub mod queries;
pub mod resources;

#[cfg(feature = "plugin")]
pub mod commands;
#[cfg(feature = "plugin")]
mod plugin;
#[cfg(feature = "plugin")]
pub mod systems;

#[cfg(feature = "plugin")]
pub use plugin::CameraPlugin;
