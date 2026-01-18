use nalgebra_glm::Vec3;
use serde::{Deserialize, Serialize};

/// Continuous rotation animation around an axis.
///
/// Entities with this component will automatically rotate each frame.
/// Useful for simple spinning animations without requiring an animation clip.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rotation {
    /// The axis of rotation (will be normalized).
    pub axis: Vec3,
    /// Angular velocity in radians per second.
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
