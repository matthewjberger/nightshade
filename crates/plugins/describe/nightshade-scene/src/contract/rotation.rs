use nalgebra_glm::Vec3;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rotation {
    pub axis: Vec3,
    pub speed: f32,
}

impl Default for Rotation {
    fn default() -> Self {
        Self {
            axis: Vec3::y(),
            speed: 1.0,
        }
    }
}
