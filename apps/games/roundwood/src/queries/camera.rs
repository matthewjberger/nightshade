use crate::data;
use crate::queries::cylinder::across_of;
use nalgebra_glm::{Quat, Vec2, Vec3};
use nightshade_camera::queries::camera_look::camera_look_rotation;

pub fn eye(focus: Vec2, lift: f32) -> Vec3 {
    Vec3::new(
        across_of(focus.x),
        lift + data::CAMERA_HEIGHT,
        data::CAMERA_REACH,
    )
}

pub fn target(focus: Vec2, lift: f32) -> Vec3 {
    Vec3::new(across_of(focus.x), lift + data::CAMERA_LOOK_LIFT, 0.0)
}

pub fn look(focus: Vec2, lift: f32) -> Quat {
    camera_look_rotation(eye(focus, lift), target(focus, lift), 0.0)
}
