use nightshade::prelude::*;
use nightshade_camera::components::{Camera, Projection};
use nightshade_camera_controller::contract::PanOrbitCamera;

pub fn frame_bounds(
    camera: Entity,
    storage: &mut nightshade::ecs::Storage,
    (min, max): (Vec3, Vec3),
) {
    let center = (min + max) * 0.5;
    let half_diagonal = ((max - min) * 0.5).norm().max(0.01);
    let field_of_view = storage
        .get::<Camera>(camera)
        .and_then(|camera| match camera.projection {
            Projection::Perspective(perspective) => Some(perspective.y_fov_rad),
            _ => None,
        })
        .unwrap_or(45.0_f32.to_radians());
    let radius = half_diagonal / (field_of_view * 0.5).sin() * 1.2;
    let near = (half_diagonal * 0.001).clamp(0.001, 0.1);

    if let Some(component) = storage.get_mut::<Camera>(camera)
        && let Projection::Perspective(perspective) = &mut component.projection
    {
        perspective.z_near = near;
    }

    if let Some(orbit) = storage.get_mut::<PanOrbitCamera>(camera) {
        orbit.target_focus = center;
        orbit.target_radius = radius;
    }
}

pub fn settle_for_preview(camera: Entity, storage: &mut nightshade::ecs::Storage) {
    use crate::data::{PREVIEW_FIT, PREVIEW_PITCH, PREVIEW_YAW};

    let Some(orbit) = storage.get_mut::<PanOrbitCamera>(camera) else {
        return;
    };
    orbit.target_radius *= PREVIEW_FIT;
    orbit.target_yaw = PREVIEW_YAW;
    orbit.target_pitch = PREVIEW_PITCH;
    orbit.focus = orbit.target_focus;
    orbit.radius = orbit.target_radius;
    orbit.yaw = orbit.target_yaw;
    orbit.pitch = orbit.target_pitch;
}

pub fn snap_axis(camera: Option<Entity>, storage: &mut nightshade::ecs::Storage, axis: [f32; 3]) {
    let Some(camera) = camera else {
        return;
    };
    let Some(orbit) = storage.get_mut::<PanOrbitCamera>(camera) else {
        return;
    };
    let direction = Vec3::new(axis[0], axis[1], axis[2]);
    let unit = if direction.norm() > 0.0001 {
        direction.normalize()
    } else {
        direction
    };
    let limit = 89.0_f32.to_radians();
    orbit.target_yaw = unit.x.atan2(unit.z);
    orbit.target_pitch = unit.y.clamp(-1.0, 1.0).asin().clamp(-limit, limit);
}
