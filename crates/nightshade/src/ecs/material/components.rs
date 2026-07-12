//! Material component definitions.

use crate::render::asset_id::MaterialId;
use serde::{Deserialize, Serialize};

/// Component referencing a material by name in the [`super::MaterialRegistry`].
///
/// The `id` field is populated when the material is resolved from the registry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, enum2schema::Schema)]
pub struct MaterialRef {
    /// Name of the material in the registry.
    pub name: String,
    /// Resolved material identifier.
    #[serde(skip)]
    #[schema(skip)]
    pub id: Option<MaterialId>,
}

impl MaterialRef {
    /// Creates a new material reference by name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            id: None,
        }
    }

    /// Creates a material reference with a pre-resolved identifier.
    pub fn with_id(name: impl Into<String>, id: MaterialId) -> Self {
        Self {
            name: name.into(),
            id: Some(id),
        }
    }
}

impl Default for MaterialRef {
    fn default() -> Self {
        Self {
            name: "Default".to_string(),
            id: None,
        }
    }
}

impl From<String> for MaterialRef {
    fn from(name: String) -> Self {
        Self { name, id: None }
    }
}

/// Per-primitive material variant table from the
/// [`KHR_materials_variants`] glTF extension. Each mapping says: when
/// any listed variant name is active, swap the entity's material to
/// the named one. Outside an active variant the entity uses
/// `default_material_name`. Variant names rather than document-scoped
/// indices are stored so multiple assets with different variant
/// orderings can coexist.
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
    /// Returns the material name to use for the supplied variant name,
    /// or `default_material_name` if no mapping covers it.
    pub fn material_for_variant(&self, variant_name: &str) -> &str {
        for mapping in &self.mappings {
            if mapping.variant_names.iter().any(|n| n == variant_name) {
                return &mapping.material_name;
            }
        }
        &self.default_material_name
    }
}

impl From<&str> for MaterialRef {
    fn from(name: &str) -> Self {
        Self {
            name: name.to_string(),
            id: None,
        }
    }
}
