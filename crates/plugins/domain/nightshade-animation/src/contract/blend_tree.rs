use nalgebra_glm::Vec2;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BlendTree {
    Clip {
        clip_index: usize,
    },
    Linear1D {
        parameter: String,
        children: Vec<Blend1DChild>,
    },
    Freeform2D {
        parameter_x: String,
        parameter_y: String,
        children: Vec<Blend2DChild>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Blend1DChild {
    pub threshold: f32,
    pub motion: BlendTree,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Blend2DChild {
    pub position: Vec2,
    pub motion: BlendTree,
}
