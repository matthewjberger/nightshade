//! The world-vs-overlay render layer tag, owned by the render layer.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderLayer(pub u32);

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
    pub const WORLD: u32 = 0;
    pub const OVERLAY: u32 = 1;

    pub fn new(layer: u32) -> Self {
        Self(layer)
    }

    pub fn is_in_layer(&self, layer: u32) -> bool {
        self.0 == layer
    }

    pub fn is_overlay(&self) -> bool {
        self.0 == Self::OVERLAY
    }
}
