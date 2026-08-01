#[cfg(feature = "contract")]
pub mod contract;

#[cfg(feature = "contract")]
pub use contract::{Config, ConfigEntry, ConfigKind, ConfigSchema, ConfigValue};

#[cfg(feature = "plugin")]
pub mod commands;
#[cfg(feature = "plugin")]
pub mod data;
#[cfg(feature = "plugin")]
mod plugin;
#[cfg(feature = "plugin")]
pub mod prelude;
#[cfg(feature = "plugin")]
pub mod queries;
#[cfg(feature = "plugin")]
pub mod resources;
#[cfg(feature = "plugin")]
pub mod systems;

#[cfg(feature = "plugin")]
pub use plugin::ConfigPlugin;
