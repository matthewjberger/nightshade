use nightshade::prelude::*;
use nightshade_camera::components::{Camera, Projection};
use nightshade_graphics::prelude::{Light, LightType};
use nightshade_platform::prelude::Window;
use nightshade_scene::contract::GlobalTransform;

use crate::data::REACH;
use crate::queries::lines::ring;
use crate::theme::{CAMERA_COLOR, LIGHT_COLOR};

pub fn marker_shape(
    storage: &nightshade::ecs::Storage,
    window: &Window,
    entity: Entity,
) -> Vec<(Vec3, Vec3, Vec4)> {
    let Some(global) = storage.get::<GlobalTransform>(entity) else {
        return Vec::new();
    };
    let origin = global.translation();
    let forward = global.forward_vector();
    let right = global.right_vector();
    let up = global.up_vector();

    if let Some(light) = storage.get::<Light>(entity) {
        return light_shape(light, origin, forward, right, up);
    }
    if let Some(camera) = storage.get::<Camera>(entity) {
        return camera_shape(camera, window, origin, forward, right, up);
    }
    Vec::new()
}

fn light_shape(
    light: &Light,
    origin: Vec3,
    forward: Vec3,
    right: Vec3,
    up: Vec3,
) -> Vec<(Vec3, Vec3, Vec4)> {
    let mut lines = Vec::new();
    match light.light_type {
        LightType::Directional => {
            let end = origin + forward * REACH;
            lines.push((origin, end, LIGHT_COLOR));
            let back = end - forward * (REACH * 0.15);
            for corner in [right, -right, up, -up] {
                lines.push((end, back + corner * (REACH * 0.06), LIGHT_COLOR));
            }
            for corner in [right + up, right - up, -right + up, -right - up] {
                let side = origin + corner * 0.25;
                lines.push((side, side + forward * REACH, LIGHT_COLOR));
            }
        }
        LightType::Point | LightType::Area => {
            let radius = light.range.max(0.05);
            lines.extend(ring(origin, right, up, radius, LIGHT_COLOR));
            lines.extend(ring(origin, right, forward, radius, LIGHT_COLOR));
            lines.extend(ring(origin, up, forward, radius, LIGHT_COLOR));
        }
        LightType::Spot => {
            let reach = light.range.max(0.05);
            let tip = origin + forward * reach;
            for (angle, color) in [
                (light.outer_cone_angle, LIGHT_COLOR),
                (
                    light.inner_cone_angle,
                    vec4(LIGHT_COLOR.x, LIGHT_COLOR.y, LIGHT_COLOR.z, 0.45),
                ),
            ] {
                let radius = reach * angle.tan();
                lines.extend(ring(tip, right, up, radius, color));
                for direction in [right, -right, up, -up] {
                    lines.push((origin, tip + direction * radius, color));
                }
            }
        }
    }
    lines
}

fn camera_shape(
    camera: &Camera,
    window: &Window,
    origin: Vec3,
    forward: Vec3,
    right: Vec3,
    up: Vec3,
) -> Vec<(Vec3, Vec3, Vec4)> {
    let aspect = window
        .cached_viewport_size
        .filter(|(_, height)| *height > 0)
        .map(|(width, height)| width as f32 / height as f32)
        .unwrap_or(16.0 / 9.0);
    let (near, far, near_half, far_half) = match &camera.projection {
        Projection::Perspective(perspective) => {
            let near = perspective.z_near.max(0.001);
            let far = perspective.z_far.unwrap_or(REACH * 4.0).min(REACH * 25.0);
            let aspect = perspective.aspect_ratio.unwrap_or(aspect);
            let half = |distance: f32| {
                let height = distance * (perspective.y_fov_rad * 0.5).tan();
                (height * aspect, height)
            };
            (near, far, half(near), half(far))
        }
        Projection::Orthographic(orthographic) => {
            let height = orthographic.y_mag;
            let width = orthographic.x_mag.unwrap_or(height * aspect);
            (
                orthographic.z_near,
                orthographic.z_far,
                (width, height),
                (width, height),
            )
        }
    };
    let corners = |distance: f32, (half_width, half_height): (f32, f32)| {
        let center = origin + forward * distance;
        [
            center - right * half_width - up * half_height,
            center + right * half_width - up * half_height,
            center + right * half_width + up * half_height,
            center - right * half_width + up * half_height,
        ]
    };
    let near_corners = corners(near, near_half);
    let far_corners = corners(far, far_half);
    let mut lines = Vec::new();
    for index in 0..4 {
        let next = (index + 1) % 4;
        lines.push((near_corners[index], near_corners[next], CAMERA_COLOR));
        lines.push((far_corners[index], far_corners[next], CAMERA_COLOR));
        lines.push((near_corners[index], far_corners[index], CAMERA_COLOR));
        lines.push((origin, near_corners[index], CAMERA_COLOR));
    }
    let top = (near_corners[2] + near_corners[3]) * 0.5;
    let peak = top + up * (near_half.1 * 0.6);
    lines.push((near_corners[2], peak, CAMERA_COLOR));
    lines.push((near_corners[3], peak, CAMERA_COLOR));
    lines
}
