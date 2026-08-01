use nightshade::prelude::*;
use nightshade_camera::components::{Camera, Projection};
use nightshade_graphics::prelude::{Light, LightType};
use nightshade_scene::contract::LocalTransform;

use crate::resources::{CameraView, LightView, TransformSnapshot, ViewerResources};

pub fn group(viewer: &ViewerResources, storage: &nightshade::ecs::Storage) -> Vec<Entity> {
    viewer
        .selection
        .selected
        .into_iter()
        .chain(viewer.selection.others.iter().copied())
        .filter(|entity| storage.is_alive(*entity))
        .collect()
}

pub fn snapshot(transform: &LocalTransform) -> TransformSnapshot {
    TransformSnapshot {
        translation: transform.translation,
        rotation: transform.rotation,
        scale: transform.scale,
    }
}

pub fn light_view(light: &Light) -> LightView {
    LightView {
        kind: match light.light_type {
            LightType::Directional => 0,
            LightType::Point => 1,
            _ => 2,
        },
        color: [light.color.x, light.color.y, light.color.z],
        intensity: light.intensity,
        range: light.range,
        inner_cone: light.inner_cone_angle.to_degrees(),
        outer_cone: light.outer_cone_angle.to_degrees(),
        cast_shadows: light.cast_shadows,
    }
}

pub fn camera_view(camera: &Camera) -> CameraView {
    match &camera.projection {
        Projection::Perspective(perspective) => CameraView {
            perspective: true,
            field_of_view: perspective.y_fov_rad.to_degrees(),
            near: perspective.z_near,
            far: perspective.z_far.unwrap_or(0.0),
        },
        Projection::Orthographic(orthographic) => CameraView {
            perspective: false,
            field_of_view: orthographic.y_mag,
            near: orthographic.z_near,
            far: orthographic.z_far,
        },
    }
}

pub fn quat_to_euler(rotation: nalgebra_glm::Quat) -> [f32; 3] {
    let coords = rotation.coords;
    let (x, y, z, w) = (coords[0], coords[1], coords[2], coords[3]);

    let sinr_cosp = 2.0 * (w * x + y * z);
    let cosr_cosp = 1.0 - 2.0 * (x * x + y * y);
    let roll = sinr_cosp.atan2(cosr_cosp);

    let sinp = 2.0 * (w * y - z * x);
    let pitch = if sinp.abs() >= 1.0 {
        std::f32::consts::FRAC_PI_2.copysign(sinp)
    } else {
        sinp.asin()
    };

    let siny_cosp = 2.0 * (w * z + x * y);
    let cosy_cosp = 1.0 - 2.0 * (y * y + z * z);
    let yaw = siny_cosp.atan2(cosy_cosp);

    [roll.to_degrees(), pitch.to_degrees(), yaw.to_degrees()]
}

pub fn euler_to_quat(degrees: [f32; 3]) -> nalgebra_glm::Quat {
    let (cos_roll, sin_roll) = half_angle(degrees[0]);
    let (cos_pitch, sin_pitch) = half_angle(degrees[1]);
    let (cos_yaw, sin_yaw) = half_angle(degrees[2]);

    let w = cos_roll * cos_pitch * cos_yaw + sin_roll * sin_pitch * sin_yaw;
    let x = sin_roll * cos_pitch * cos_yaw - cos_roll * sin_pitch * sin_yaw;
    let y = cos_roll * sin_pitch * cos_yaw + sin_roll * cos_pitch * sin_yaw;
    let z = cos_roll * cos_pitch * sin_yaw - sin_roll * sin_pitch * cos_yaw;
    nalgebra_glm::Quat::new(w, x, y, z)
}

fn half_angle(degrees: f32) -> (f32, f32) {
    let half = degrees.to_radians() * 0.5;
    (half.cos(), half.sin())
}
