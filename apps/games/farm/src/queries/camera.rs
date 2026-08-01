use crate::theme;
use nalgebra_glm::{Vec2, Vec3};

pub fn eye_offset(yaw: f32) -> Vec3 {
    Vec3::new(
        yaw.sin() * theme::CAMERA_REACH,
        theme::CAMERA_HEIGHT,
        yaw.cos() * theme::CAMERA_REACH,
    )
}

pub fn screen_right(yaw: f32) -> Vec2 {
    Vec2::new(yaw.cos(), -yaw.sin())
}

pub fn screen_down(yaw: f32) -> Vec2 {
    Vec2::new(yaw.sin(), yaw.cos())
}

pub fn ground_step(yaw: f32, input: Vec2) -> Vec2 {
    screen_right(yaw) * input.x + screen_down(yaw) * input.y
}
