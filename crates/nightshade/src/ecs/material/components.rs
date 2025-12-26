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
    pub normal_texture: Option<String>,
    #[serde(default = "default_normal_scale")]
    pub normal_scale: f32,
    #[serde(default)]
    pub normal_map_flip_y: bool,
    #[serde(default)]
    pub normal_map_two_component: bool,
    pub metallic_roughness_texture: Option<String>,
    pub occlusion_texture: Option<String>,
    #[serde(default = "default_occlusion_strength")]
    pub occlusion_strength: f32,
    pub roughness: f32,
    pub metallic: f32,
    pub unlit: bool,
    #[serde(default = "default_uv_scale")]
    pub uv_scale: [f32; 2],
}

fn default_uv_scale() -> [f32; 2] {
    [1.0, 1.0]
}

fn default_normal_scale() -> f32 {
    1.0
}

fn default_occlusion_strength() -> f32 {
    1.0
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
            normal_texture: None,
            normal_scale: 1.0,
            normal_map_flip_y: false,
            normal_map_two_component: false,
            metallic_roughness_texture: None,
            occlusion_texture: None,
            occlusion_strength: 1.0,
            roughness: 0.5,
            metallic: 0.0,
            unlit: false,
            uv_scale: [1.0, 1.0],
        }
    }
}
