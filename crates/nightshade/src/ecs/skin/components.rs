use nalgebra_glm::Mat4;
use nightshade_ecs::Entity;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Skin {
    pub joints: Vec<Entity>,
    #[serde(skip, default = "default_inverse_bind_matrices")]
    pub inverse_bind_matrices: Arc<Vec<Mat4>>,
    pub name: Option<String>,
}

fn default_inverse_bind_matrices() -> Arc<Vec<Mat4>> {
    Arc::new(Vec::new())
}

impl Default for Skin {
    fn default() -> Self {
        Self {
            joints: Vec::new(),
            inverse_bind_matrices: Arc::new(Vec::new()),
            name: None,
        }
    }
}

impl Skin {
    pub fn new(joints: Vec<Entity>, inverse_bind_matrices: Vec<Mat4>) -> Self {
        Self {
            joints,
            inverse_bind_matrices: Arc::new(inverse_bind_matrices),
            name: None,
        }
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Joint {
    pub index_in_skin: usize,
    pub name: Option<String>,
}

impl Joint {
    pub fn new(index_in_skin: usize) -> Self {
        Self {
            index_in_skin,
            name: None,
        }
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
}
