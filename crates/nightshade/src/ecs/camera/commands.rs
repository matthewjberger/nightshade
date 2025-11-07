use crate::ecs::world::{Entity, Vec3, World, components::*};

pub fn spawn_camera(world: &mut World, position: Vec3, name: String) -> Entity {
    let cameras = world.spawn_entities(
        crate::ecs::world::NAME
            | crate::ecs::world::LOCAL_TRANSFORM
            | crate::ecs::world::LOCAL_TRANSFORM_DIRTY
            | crate::ecs::world::GLOBAL_TRANSFORM
            | crate::ecs::world::CAMERA,
        1,
    );

    let camera = cameras[0];

    if let Some(camera_name) = world.get_name_mut(camera) {
        *camera_name = Name(name);
    }

    if let Some(local_transform) = world.get_local_transform_mut(camera) {
        local_transform.translation = position;
    }

    if let Some(camera_component) = world.get_camera_mut(camera) {
        *camera_component = Camera {
            projection: Projection::Perspective(PerspectiveCamera {
                aspect_ratio: None,
                y_fov_rad: 45.0_f32.to_radians(),
                z_far: None,
                z_near: 0.01,
            }),
            smoothing: Some(Smoothing::default()),
        };
    }

    camera
}
