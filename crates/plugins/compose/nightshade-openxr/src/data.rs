use nalgebra_glm::{Quat, Vec3};
use openxr as xr;

pub fn to_vec3(position: xr::Vector3f) -> Vec3 {
    nalgebra_glm::vec3(position.x, position.y, position.z)
}

pub fn to_quat(orientation: xr::Quaternionf) -> Quat {
    nalgebra_glm::quat(orientation.x, orientation.y, orientation.z, orientation.w)
}
