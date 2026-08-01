pub mod contract;

#[cfg(feature = "plugin")]
mod plugin;
#[cfg(feature = "plugin")]
pub mod systems;

pub use contract::render_data::DecalInstance;
#[cfg(feature = "plugin")]
pub use plugin::DecalsPlugin;
