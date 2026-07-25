//! Camera component definitions.

mod orthographic;
mod pan_orbit;
mod perspective;
mod projection_override;
mod render_target;
mod smoothing;
mod third_person;
mod viewport_shading;

pub use orthographic::*;
pub use pan_orbit::*;
pub use perspective::*;
pub use projection_override::*;
pub use render_target::*;
pub use smoothing::*;
pub use third_person::*;
pub use viewport_shading::*;

use nalgebra_glm::Mat4;

/// Primary camera component defining view projection.
///
/// Attach to an entity with a transform to create a viewpoint. The active camera
/// is determined by the rendering system (typically the first entity with this component).
#[derive(
    Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize, enum2schema::Schema,
)]
pub struct Camera {
    /// The projection mode (perspective or orthographic).
    pub projection: Projection,
    /// Optional input smoothing for camera movement.
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

/// Projection mode for a camera.
#[derive(
    Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize, enum2schema::Schema,
)]
pub enum Projection {
    /// Perspective projection with foreshortening (3D games, realistic rendering).
    Perspective(PerspectiveCamera),
    /// Orthographic projection without foreshortening (2D games, CAD, isometric views).
    Orthographic(OrthographicCamera),
}

impl Projection {
    /// Returns the projection matrix using default or stored aspect ratio.
    pub fn matrix(&self) -> Mat4 {
        match self {
            Projection::Perspective(camera) => camera.matrix(),
            Projection::Orthographic(camera) => camera.matrix(),
        }
    }

    /// Returns the projection matrix with the specified aspect ratio.
    ///
    /// For perspective cameras, this overrides the stored aspect ratio.
    /// For orthographic cameras, aspect ratio is ignored.
    pub fn matrix_with_aspect(&self, aspect_ratio: f32) -> Mat4 {
        match self {
            Projection::Perspective(camera) => camera.matrix_with_aspect(aspect_ratio),
            Projection::Orthographic(camera) => camera.matrix(),
        }
    }
}
