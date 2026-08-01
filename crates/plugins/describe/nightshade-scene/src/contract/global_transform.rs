use nalgebra_glm::{Mat4, Vec3};

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct GlobalTransform(pub Mat4);

impl Default for GlobalTransform {
    fn default() -> Self {
        Self(Mat4::identity())
    }
}

impl GlobalTransform {
    pub fn translation(&self) -> Vec3 {
        nalgebra_glm::vec3(self.0[(0, 3)], self.0[(1, 3)], self.0[(2, 3)])
    }

    pub fn right_vector(&self) -> Vec3 {
        extract_right_vector(&self.0)
    }

    pub fn up_vector(&self) -> Vec3 {
        extract_up_vector(&self.0)
    }

    pub fn forward_vector(&self) -> Vec3 {
        extract_forward_vector(&self.0)
    }
}

pub(super) fn extract_right_vector(matrix: &Mat4) -> Vec3 {
    nalgebra_glm::vec3(matrix[(0, 0)], matrix[(1, 0)], matrix[(2, 0)])
}

pub(super) fn extract_up_vector(matrix: &Mat4) -> Vec3 {
    nalgebra_glm::vec3(matrix[(0, 1)], matrix[(1, 1)], matrix[(2, 1)])
}

pub(super) fn extract_forward_vector(matrix: &Mat4) -> Vec3 {
    nalgebra_glm::vec3(-matrix[(0, 2)], -matrix[(1, 2)], -matrix[(2, 2)])
}
