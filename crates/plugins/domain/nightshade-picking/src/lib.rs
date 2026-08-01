pub mod contract;

#[cfg(feature = "plugin")]
pub mod plugin;
#[cfg(feature = "plugin")]
pub mod queries;
#[cfg(feature = "plugin")]
pub mod resources;
#[cfg(feature = "plugin")]
pub mod systems;

pub use contract::{PickingOptions, PickingRay, PickingResult};

#[cfg(feature = "plugin")]
pub use plugin::PickingPlugin;
#[cfg(feature = "plugin")]
pub use resources::{PickWhen, Picked};

#[cfg(feature = "plugin")]
pub use queries::{
    CameraRows, CameraView, PickableRows, ScreenArea, pick_closest_entity, pick_entities,
    pick_entities_in_frustum, screen_ray,
};
