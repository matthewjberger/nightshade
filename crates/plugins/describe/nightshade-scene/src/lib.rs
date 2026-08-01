pub mod contract;

#[cfg(feature = "plugin")]
pub mod commands;
#[cfg(feature = "plugin")]
pub mod plugin;
#[cfg(feature = "plugin")]
pub mod queries;
#[cfg(feature = "plugin")]
pub mod resources;
#[cfg(feature = "plugin")]
pub mod systems;

#[cfg(feature = "plugin")]
pub use plugin::{ScenePlugin, install, register_frame_systems};

pub mod prelude {
    pub use crate::contract::*;

    #[cfg(feature = "plugin")]
    pub use crate::commands::entity_registry::ensure_guid;
    #[cfg(feature = "plugin")]
    pub use crate::commands::transform::{
        propagate_subtree, setup_entity_transforms, update_parent,
    };
    #[cfg(feature = "plugin")]
    pub use crate::queries::*;
    #[cfg(feature = "plugin")]
    pub use crate::resources::{EntityRegistry, TransformState};
    #[cfg(feature = "plugin")]
    pub use crate::systems::{expire_lifetimes_system, update_global_transforms_system};
}
