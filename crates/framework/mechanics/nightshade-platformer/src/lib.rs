pub mod commands;
pub mod components;
pub mod plugin;
pub mod prelude;
pub mod queries;
pub mod resources;
pub mod systems;

pub use components::{Airborne, Coyote, JumpAsked};
pub use plugin::PlatformerPlugin;
pub use resources::PlatformerSettings;
