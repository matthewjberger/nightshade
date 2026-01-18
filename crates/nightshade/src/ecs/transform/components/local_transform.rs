use nalgebra_glm::{Mat4, Quat, Vec3, quat_to_mat4};
use serde::{Deserialize, Serialize};

/// Position, rotation, and scale relative to the parent entity (or world origin).
///
/// This is the primary component for positioning entities in the scene. Changes to
/// this component are propagated through the hierarchy to compute [`super::GlobalTransform`].
///
/// The transformation is applied in TRS order: Translation × Rotation × Scale.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct LocalTransform {
    /// Position offset from parent origin.
    pub translation: Vec3,
    /// Orientation as a unit quaternion.
    pub rotation: Quat,
    /// Non-uniform scale factors along each axis.
    pub scale: Vec3,
}

impl Default for LocalTransform {
    fn default() -> Self {
        Self {
            translation: Vec3::zeros(),
            rotation: Quat::identity(),
            scale: Vec3::new(1.0, 1.0, 1.0),
        }
    }
}

impl LocalTransform {
    /// Creates a new transform with explicit translation, rotation, and scale.
    pub fn new(translation: Vec3, rotation: Quat, scale: Vec3) -> Self {
        Self {
            translation,
            rotation,
            scale,
        }
    }

    /// Creates a transform with only translation, using identity rotation and unit scale.
    pub fn from_translation(translation: Vec3) -> Self {
        Self {
            translation,
            ..Default::default()
        }
    }

    /// Creates a transform with only rotation, using zero translation and unit scale.
    pub fn from_rotation(rotation: Quat) -> Self {
        Self {
            rotation,
            ..Default::default()
        }
    }

    /// Creates a transform with only scale, using zero translation and identity rotation.
    pub fn from_scale(scale: Vec3) -> Self {
        Self {
            scale,
            ..Default::default()
        }
    }

    /// Computes the 4x4 transformation matrix (Translation × Rotation × Scale).
    pub fn as_matrix(&self) -> Mat4 {
        let translation_matrix = nalgebra_glm::translation(&self.translation);
        let rotation_matrix = quat_to_mat4(&self.rotation.normalize());
        let scale_matrix = nalgebra_glm::scaling(&self.scale);
        translation_matrix * rotation_matrix * scale_matrix
    }

    /// Returns the local +X axis direction.
    pub fn right_vector(&self) -> Vec3 {
        super::extract_right_vector(&self.as_matrix())
    }

    /// Returns the local +Y axis direction.
    pub fn up_vector(&self) -> Vec3 {
        super::extract_up_vector(&self.as_matrix())
    }

    /// Returns the local -Z axis direction (forward in a right-handed coordinate system).
    pub fn forward_vector(&self) -> Vec3 {
        super::extract_forward_vector(&self.as_matrix())
    }
}
