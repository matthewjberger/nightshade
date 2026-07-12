pub mod atlas;
pub mod cascade;
mod culling;
mod node;
mod pass;
pub mod types;

pub use pass::ShadowDepthPass;
pub use types::{
    CASCADE_ATLAS_SIZE, CASCADE_SLOT_RESOLUTION, CASCADE_SPLIT_DISTANCES, CascadeData,
    MAX_POINT_LIGHT_SHADOWS, NUM_SHADOW_CASCADES, PointLightShadowData, SpotlightShadowData,
    SpotlightShadowSlot, scale_cascade_splits,
};
