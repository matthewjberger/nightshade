//! Cascaded directional shadows plus a spotlight and area-light shadow atlas.

pub mod atlas;
/// Cascade split math and reverse-Z orthographic light projections.
pub mod cascade;
mod culling;
mod node;
mod pass;
/// Shadow constants and the GPU cascade, spotlight, and point-light data types.
pub mod types;

pub use pass::ShadowDepthPass;
pub use types::{
    CASCADE_ATLAS_SIZE, CASCADE_REFERENCE_NEAR, CASCADE_SLOT_RESOLUTION, CASCADE_SPLIT_DISTANCES,
    CascadeData, MAX_POINT_LIGHT_SHADOWS, NUM_SHADOW_CASCADES, PointLightShadowData,
    SpotlightShadowData, SpotlightShadowSlot, scale_cascade_splits,
};
