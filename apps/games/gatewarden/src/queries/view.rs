use crate::theme::{BURST_DISTANCE, CAMERA_FOV_RADIANS};
use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::*;

pub fn screen_to_world(camera: &LocalTransform, window: &Window, point: Vec2) -> Vec3 {
    let forward = nalgebra_glm::quat_rotate_vec3(&camera.rotation, &Vec3::new(0.0, 0.0, -1.0));
    let right = nalgebra_glm::quat_rotate_vec3(&camera.rotation, &Vec3::new(1.0, 0.0, 0.0));
    let up = nalgebra_glm::quat_rotate_vec3(&camera.rotation, &Vec3::new(0.0, 1.0, 0.0));
    let (width, height) = window.cached_viewport_size.unwrap_or((1600, 900));
    let width = width.max(1) as f32;
    let height = height.max(1) as f32;
    let half_height = BURST_DISTANCE * (CAMERA_FOV_RADIANS * 0.5).tan();
    let half_width = half_height * width / height;
    let normalized_x = point.x / width - 0.5;
    let normalized_y = 0.5 - point.y / height;
    camera.translation
        + forward * BURST_DISTANCE
        + right * (normalized_x * 2.0 * half_width)
        + up * (normalized_y * 2.0 * half_height)
}
