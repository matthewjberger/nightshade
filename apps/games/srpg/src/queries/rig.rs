use crate::resources::CameraRig;
use nalgebra_glm::Vec2;

pub fn right(rig: &CameraRig) -> Vec2 {
    Vec2::new(rig.yaw.cos(), -rig.yaw.sin())
}

pub fn snapped_forward(rig: &CameraRig) -> (i32, i32) {
    match rig.yaw_index.rem_euclid(4) {
        0 => (0, -1),
        1 => (-1, 0),
        2 => (0, 1),
        _ => (1, 0),
    }
}

pub fn snapped_right(rig: &CameraRig) -> (i32, i32) {
    let (forward_column, forward_row) = snapped_forward(rig);
    (-forward_row, forward_column)
}
