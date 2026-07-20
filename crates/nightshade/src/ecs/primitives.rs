//! Small primitive component types and shared helpers that don't justify
//! their own module. Grouped here to keep the top-level `ecs/` tree
//! shallow.

use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(default)]
pub struct Name(pub String);

impl enum2schema::Schema for Name {
    fn schema() -> enum2schema::serde_json::Value {
        enum2schema::serde_json::json!({ "type": "string" })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, enum2schema::Schema)]
#[serde(default)]
pub struct Visibility {
    pub visible: bool,
}

impl Default for Visibility {
    fn default() -> Self {
        Self { visible: true }
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CastsShadow;

impl enum2schema::Schema for CastsShadow {
    fn schema() -> enum2schema::serde_json::Value {
        enum2schema::serde_json::json!({ "type": "null" })
    }
}

/// Stable engine-side identity for entities that round-trip through a
/// save file or that other entities cross-reference. Opt-in: omit on
/// transient entities (particles, projectiles, popups). Engine mints
/// values from `world.res::<crate::ecs::entity_registry::EntityRegistry>().next_guid`, which is itself
/// persisted in the scene file so freshly-loaded scenes don't collide
/// with the writer's ids.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Guid(pub u64);

/// Bitmask of "culling layers" this entity belongs to. Independent
/// from [`RenderLayer`] (which selects WORLD vs OVERLAY rendering).
/// Cameras with a [`CameraCullingMask`] component skip entities
/// whose `(entity_layers & camera_mask) == 0`. The default for
/// entities without the component is bit 0 set (layer 0), and the
/// default for cameras without the component is all bits set, so
/// per-camera culling is purely opt-in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CullingMask(pub u32);

impl Default for CullingMask {
    fn default() -> Self {
        Self(1)
    }
}

impl CullingMask {
    pub const DEFAULT_LAYER: u32 = 1;

    pub fn new(mask: u32) -> Self {
        Self(mask)
    }

    pub fn add_layer(&mut self, layer_index: u32) {
        self.0 |= 1u32 << layer_index;
    }

    pub fn remove_layer(&mut self, layer_index: u32) {
        self.0 &= !(1u32 << layer_index);
    }

    pub fn intersects(self, other: CullingMask) -> bool {
        (self.0 & other.0) != 0
    }
}

impl enum2schema::Schema for CullingMask {
    fn schema() -> enum2schema::serde_json::Value {
        enum2schema::serde_json::json!({ "type": "integer" })
    }
}

/// Camera-side counterpart to [`CullingMask`]. The renderer
/// resolves this into `EffectiveShading::culling_mask` per frame
/// and the mesh / skinned-mesh prepare passes skip any entity whose
/// own `CullingMask` doesn't intersect the camera's mask. Cameras
/// without this component render every entity (mask = `!0`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CameraCullingMask(pub u32);

impl Default for CameraCullingMask {
    fn default() -> Self {
        Self(!0)
    }
}

impl CameraCullingMask {
    pub fn new(mask: u32) -> Self {
        Self(mask)
    }

    pub fn intersects(self, layer: CullingMask) -> bool {
        (self.0 & layer.0) != 0
    }
}

impl enum2schema::Schema for CameraCullingMask {
    fn schema() -> enum2schema::serde_json::Value {
        enum2schema::serde_json::json!({ "type": "integer" })
    }
}

pub use nightshade_renderer::easing::EasingFunction;
