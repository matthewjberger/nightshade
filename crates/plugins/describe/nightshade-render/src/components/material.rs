use crate::data::asset_id::{DEFAULT_MATERIAL_ID, MaterialId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MaterialRef {
    pub id: MaterialId,
}

impl MaterialRef {
    pub fn new(id: MaterialId) -> Self {
        Self { id }
    }
}

impl Default for MaterialRef {
    fn default() -> Self {
        Self {
            id: DEFAULT_MATERIAL_ID,
        }
    }
}

impl From<MaterialId> for MaterialRef {
    fn from(id: MaterialId) -> Self {
        Self { id }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MaterialVariants {
    pub default_material_name: String,
    pub mappings: Vec<MaterialVariantMapping>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaterialVariantMapping {
    pub variant_names: Vec<String>,
    pub material_name: String,
}

impl MaterialVariants {
    pub fn material_for_variant(&self, variant_name: &str) -> &str {
        for mapping in &self.mappings {
            if mapping.variant_names.iter().any(|n| n == variant_name) {
                return &mapping.material_name;
            }
        }
        &self.default_material_name
    }
}
