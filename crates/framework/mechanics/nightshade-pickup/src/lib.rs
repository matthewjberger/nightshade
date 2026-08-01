pub mod components;
pub mod data;
pub mod plugin;
pub mod prelude;
pub mod systems;

pub use components::{Collector, Middle, Pickup, PickupCount, Taken};
pub use data::PickedUp;
pub use plugin::PickupPlugin;
