#[cfg(feature = "contract")]
pub mod contract;

#[cfg(feature = "plugin")]
pub mod data;
#[cfg(feature = "plugin")]
mod plugin;
#[cfg(feature = "plugin")]
pub mod resources;
#[cfg(feature = "plugin")]
pub mod systems;

#[cfg(feature = "contract")]
pub use contract::render_data::{
    SdfChunkKey, SdfClipmap, SdfCollisionMesh, SdfDebugView, SdfEditInstance, SdfFrame,
};
#[cfg(feature = "plugin")]
pub use plugin::SdfPlugin;

#[cfg(feature = "plugin")]
pub mod prelude {
    pub use crate::contract::render_data::{
        SdfChunkKey, SdfClipmap, SdfCollisionMesh, SdfDebugView, SdfEditInstance, SdfFrame,
    };
    pub use crate::contract::{
        SdfBlend, SdfCollides, SdfCollisionFocus, SdfMetal, SdfOperation, SdfOrder, SdfRoughness,
        SdfShape, SdfTint,
    };
    pub use crate::plugin::SdfPlugin;
    pub use crate::resources::SdfCollision;
}
