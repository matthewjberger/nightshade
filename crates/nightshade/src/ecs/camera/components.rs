mod orthographic;
mod perspective;
mod smoothing;

pub use orthographic::*;
pub use perspective::*;
pub use smoothing::*;

use nalgebra_glm::Mat4;

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Camera {
    pub projection: Projection,
    pub smoothing: Option<Smoothing>,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            projection: Projection::Perspective(PerspectiveCamera::default()),
            smoothing: Some(Smoothing::default()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Projection {
    Perspective(PerspectiveCamera),
    Orthographic(OrthographicCamera),
}

impl Projection {
    pub fn matrix(&self) -> Mat4 {
        match self {
            Projection::Perspective(camera) => camera.matrix(),
            Projection::Orthographic(camera) => camera.matrix(),
        }
    }
}
