use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AlphaMode {
    #[default]
    Opaque,
    Mask,
    Blend,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
        }
    }
}
