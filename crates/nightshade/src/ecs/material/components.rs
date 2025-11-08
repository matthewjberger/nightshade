use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlphaMode {
    Opaque,
    Mask,
    Blend,
}

impl Default for AlphaMode {
    fn default() -> Self {
        Self::Opaque
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Material {
    pub base_color: [f32; 4],
    pub alpha_mode: AlphaMode,
    pub alpha_cutoff: f32,
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
            alpha_mode: AlphaMode::Opaque,
            alpha_cutoff: 0.5,
        }
    }
}
