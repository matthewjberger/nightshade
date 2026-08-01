#[cfg(feature = "contract")]
pub mod contract;

#[cfg(feature = "plugin")]
pub mod commands;
#[cfg(feature = "plugin")]
#[cfg(feature = "plugin")]
mod plugin;
#[cfg(feature = "plugin")]
mod systems;

#[cfg(feature = "plugin")]
pub use plugin::{ParticlePlugin, install, register_frame_systems};
