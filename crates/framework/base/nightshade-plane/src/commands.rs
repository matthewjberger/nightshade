use crate::data::plane::Plane;
use nalgebra_glm::Vec3;

pub fn move_across(plane: Plane, spot: &mut Vec3, step: Vec3) {
    spot.x += step.x;
    match plane {
        Plane::Ground => spot.z += step.z,
        Plane::Side => spot.y += step.y,
    }
}
