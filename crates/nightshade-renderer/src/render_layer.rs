//! The world-vs-overlay render layer tag, owned by the render layer.

use serde::{Deserialize, Serialize};

/// Render layer tag selecting whether an entity draws in the world or as an
/// overlay.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderLayer(
    /// Layer index.
    pub u32,
);

impl enum2schema::Schema for RenderLayer {
    fn schema() -> enum2schema::serde_json::Value {
        enum2schema::serde_json::json!({ "type": "integer" })
    }
}

impl Default for RenderLayer {
    fn default() -> Self {
        Self(Self::WORLD)
    }
}

impl RenderLayer {
    /// The main world render layer.
    pub const WORLD: u32 = 0;
    /// The overlay render layer, drawn on top of the world.
    pub const OVERLAY: u32 = 1;

    /// Wraps a raw layer index.
    pub fn new(layer: u32) -> Self {
        Self(layer)
    }

    /// Returns whether this tag is in `layer`.
    pub fn is_in_layer(&self, layer: u32) -> bool {
        self.0 == layer
    }

    /// Returns whether this is the overlay layer.
    pub fn is_overlay(&self) -> bool {
        self.0 == Self::OVERLAY
    }
}
