pub mod cascade;
mod culling;
mod node;
mod pass;
pub mod types;

pub use pass::ShadowDepthPass;
pub use types::{
    CASCADE_ATLAS_SIZE, CASCADE_SLOT_RESOLUTION, CASCADE_SPLIT_DISTANCES, MAX_POINT_LIGHT_SHADOWS,
    MAX_SPOTLIGHT_SHADOWS, NUM_SHADOW_CASCADES, PointLightShadowData, SPOTLIGHT_ATLAS_SIZE,
    SpotlightShadowData, point_shadow_slots,
};
