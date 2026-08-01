#[cfg(feature = "contract")]
pub mod commands;
#[cfg(feature = "contract")]
pub mod data;
#[cfg(feature = "contract")]
pub mod prelude;
#[cfg(feature = "contract")]
pub mod queries;
#[cfg(feature = "contract")]
pub mod resources;

#[cfg(feature = "plugin")]
pub mod plugin;
#[cfg(feature = "plugin")]
pub mod systems;
#[cfg(feature = "plugin")]
pub use plugin::GraphicsPlugin;
