pub mod contract;

#[cfg(feature = "plugin")]
pub mod data;
#[cfg(feature = "plugin")]
mod gpu;
#[cfg(feature = "plugin")]
mod plugin;
#[cfg(feature = "plugin")]
pub mod resources;
#[cfg(feature = "plugin")]
mod runner;
#[cfg(feature = "plugin")]
mod systems;

#[cfg(feature = "plugin")]
pub use plugin::OpenXrPlugin;
