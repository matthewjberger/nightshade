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
    #[serde(default)]
    pub base_texture_uv_set: u32,
    pub emissive_texture: Option<String>,
    #[serde(default)]
    pub emissive_texture_uv_set: u32,
    pub normal_texture: Option<String>,
    #[serde(default)]
    pub normal_texture_uv_set: u32,
    #[serde(default = "default_normal_scale")]
    pub normal_scale: f32,
    #[serde(default)]
    pub normal_map_flip_y: bool,
    #[serde(default)]
    pub normal_map_two_component: bool,
    pub metallic_roughness_texture: Option<String>,
    #[serde(default)]
    pub metallic_roughness_texture_uv_set: u32,
    pub occlusion_texture: Option<String>,
    #[serde(default)]
    pub occlusion_texture_uv_set: u32,
    #[serde(default = "default_occlusion_strength")]
    pub occlusion_strength: f32,
    pub roughness: f32,
    pub metallic: f32,
    pub unlit: bool,
    #[serde(default)]
    pub double_sided: bool,
    #[serde(default = "default_uv_scale")]
    pub uv_scale: [f32; 2],
    #[serde(default)]
    pub transmission_factor: f32,
    #[serde(default)]
    pub transmission_texture: Option<String>,
    #[serde(default)]
    pub transmission_texture_uv_set: u32,
    #[serde(default)]
    pub thickness: f32,
    #[serde(default)]
    pub thickness_texture: Option<String>,
    #[serde(default)]
    pub thickness_texture_uv_set: u32,
    #[serde(default = "default_attenuation_color")]
    pub attenuation_color: [f32; 3],
    #[serde(default)]
    pub attenuation_distance: f32,
    #[serde(default = "default_ior")]
    pub ior: f32,
    #[serde(default = "default_specular_factor")]
    pub specular_factor: f32,
    #[serde(default = "default_specular_color_factor")]
    pub specular_color_factor: [f32; 3],
    #[serde(default)]
    pub specular_texture: Option<String>,
    #[serde(default)]
    pub specular_texture_uv_set: u32,
    #[serde(default)]
    pub specular_color_texture: Option<String>,
    #[serde(default)]
    pub specular_color_texture_uv_set: u32,
    #[serde(default = "default_emissive_strength")]
    pub emissive_strength: f32,
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

fn default_attenuation_color() -> [f32; 3] {
    [1.0, 1.0, 1.0]
}

fn default_ior() -> f32 {
    1.5
}

fn default_specular_factor() -> f32 {
    1.0
}

fn default_specular_color_factor() -> [f32; 3] {
    [1.0, 1.0, 1.0]
}

fn default_emissive_strength() -> f32 {
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
            base_texture_uv_set: 0,
            emissive_texture: None,
            emissive_texture_uv_set: 0,
            normal_texture: None,
            normal_texture_uv_set: 0,
            normal_scale: 1.0,
            normal_map_flip_y: false,
            normal_map_two_component: false,
            metallic_roughness_texture: None,
            metallic_roughness_texture_uv_set: 0,
            occlusion_texture: None,
            occlusion_texture_uv_set: 0,
            occlusion_strength: 1.0,
            roughness: 0.5,
            metallic: 0.0,
            unlit: false,
            double_sided: false,
            uv_scale: [1.0, 1.0],
            transmission_factor: 0.0,
            transmission_texture: None,
            transmission_texture_uv_set: 0,
            thickness: 0.0,
            thickness_texture: None,
            thickness_texture_uv_set: 0,
            attenuation_color: [1.0, 1.0, 1.0],
            attenuation_distance: 0.0,
            ior: 1.5,
            specular_factor: 1.0,
            specular_color_factor: [1.0, 1.0, 1.0],
            specular_texture: None,
            specular_texture_uv_set: 0,
            specular_color_texture: None,
            specular_color_texture_uv_set: 0,
            emissive_strength: 1.0,
        }
    }
}
