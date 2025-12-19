use crate::ecs::asset_id::MaterialId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaterialRef {
    pub name: String,
    #[serde(skip)]
    pub id: Option<MaterialId>,
}

impl MaterialRef {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            id: None,
        }
    }

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

impl From<&str> for MaterialRef {
    fn from(name: &str) -> Self {
        Self {
            name: name.to_string(),
            id: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, Hash)]
pub enum AlphaMode {
    #[default]
    Opaque,
    Mask,
    Blend,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Material {
    pub base_color: [f32; 4],
    pub emissive_factor: [f32; 3],
    pub alpha_mode: AlphaMode,
    pub alpha_cutoff: f32,
    pub base_texture: Option<String>,
    pub emissive_texture: Option<String>,
    pub roughness: f32,
    pub metallic: f32,
    pub unlit: bool,
    #[serde(default = "default_uv_scale")]
    pub uv_scale: [f32; 2],
}

fn default_uv_scale() -> [f32; 2] {
    [1.0, 1.0]
}

impl Material {
    pub fn is_transparent(&self) -> bool {
        matches!(self.alpha_mode, AlphaMode::Mask | AlphaMode::Blend)
    }
}

impl Default for Material {
    fn default() -> Self {
        Self {
            base_color: [0.7, 0.7, 0.7, 1.0],
            emissive_factor: [0.0, 0.0, 0.0],
            alpha_mode: AlphaMode::Opaque,
            alpha_cutoff: 0.5,
            base_texture: None,
            emissive_texture: None,
            roughness: 0.5,
            metallic: 0.0,
            unlit: false,
            uv_scale: [1.0, 1.0],
        }
    }
}
