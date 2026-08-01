#[cfg(feature = "contract")]
pub mod contract;

#[cfg(feature = "plugin")]
pub mod commands;
#[cfg(feature = "plugin")]
mod data;
#[cfg(feature = "plugin")]
pub mod plugin;
#[cfg(feature = "plugin")]
pub mod prelude;
#[cfg(feature = "plugin")]
pub mod queries;
#[cfg(feature = "plugin")]
pub mod resources;
#[cfg(feature = "plugin")]
pub mod systems;

#[cfg(feature = "contract")]
pub use contract::{
    Action, Actions, Binding, Bindings, DEFAULT_PROFILE, EngineAction, Pointer, Press, Stick,
};
#[cfg(feature = "plugin")]
pub use plugin::InputPlugin;
