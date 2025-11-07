mod local_transform;
mod local_transform_dirty;
mod parent;

pub use local_transform::*;
pub use local_transform_dirty::*;
pub use parent::*;

use nalgebra_glm::{Mat4, Vec3};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GlobalTransform(pub Mat4);

impl Default for GlobalTransform {
    fn default() -> Self {
        Self(Mat4::identity())
    }
}

impl GlobalTransform {
    pub fn position(&self) -> Vec3 {
        self.0.column(3).xyz()
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

fn extract_right_vector(matrix: &Mat4) -> Vec3 {
    matrix.column(0).xyz()
}

fn extract_up_vector(matrix: &Mat4) -> Vec3 {
    matrix.column(1).xyz()
}

fn extract_forward_vector(matrix: &Mat4) -> Vec3 {
    -matrix.column(2).xyz()
}
