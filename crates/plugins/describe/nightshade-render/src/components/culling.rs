use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CastsShadow;

impl enum2schema::Schema for CastsShadow {
    fn schema() -> enum2schema::serde_json::Value {
        enum2schema::serde_json::json!({ "type": "null" })
    }
}

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
