use nalgebra_glm::{Mat4, Quat, Vec3, quat_to_mat4};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LocalTransform {
    pub translation: Vec3,
    pub rotation: Quat,
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
    pub fn new(translation: Vec3, rotation: Quat, scale: Vec3) -> Self {
        Self {
            translation,
            rotation,
            scale,
        }
    }

    pub fn from_translation(translation: Vec3) -> Self {
        Self {
            translation,
            ..Default::default()
        }
    }

    pub fn from_rotation(rotation: Quat) -> Self {
        Self {
            rotation,
            ..Default::default()
        }
    }

    pub fn from_scale(scale: Vec3) -> Self {
        Self {
            scale,
            ..Default::default()
        }
    }

    pub fn as_matrix(&self) -> Mat4 {
        let translation_matrix = nalgebra_glm::translation(&self.translation);
        let rotation_matrix = quat_to_mat4(&self.rotation.normalize());
        let scale_matrix = nalgebra_glm::scaling(&self.scale);
        translation_matrix * rotation_matrix * scale_matrix
    }

    pub fn right_vector(&self) -> Vec3 {
        super::extract_right_vector(&self.as_matrix())
    }

    pub fn up_vector(&self) -> Vec3 {
        super::extract_up_vector(&self.as_matrix())
    }

    pub fn forward_vector(&self) -> Vec3 {
        super::extract_forward_vector(&self.as_matrix())
    }
}
