use crate::ecs::camera::components::compute_pan_orbit_transform;
use crate::ecs::world::{Entity, PAN_ORBIT_CAMERA, Vec2, Vec3, World, components::*};

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

pub fn spawn_pan_orbit_camera(
    world: &mut World,
    focus: Vec3,
    radius: f32,
    yaw: f32,
    pitch: f32,
    name: String,
) -> Entity {
    let cameras = world.spawn_entities(
        crate::ecs::world::NAME
            | crate::ecs::world::LOCAL_TRANSFORM
            | crate::ecs::world::LOCAL_TRANSFORM_DIRTY
            | crate::ecs::world::GLOBAL_TRANSFORM
            | crate::ecs::world::CAMERA
            | PAN_ORBIT_CAMERA,
        1,
    );

    let camera = cameras[0];

    if let Some(camera_name) = world.get_name_mut(camera) {
        *camera_name = Name(name);
    }

    let pan_orbit = PanOrbitCamera::new(focus, radius).with_yaw_pitch(yaw, pitch);
    let (position, rotation) = compute_pan_orbit_transform(focus, yaw, pitch, radius);

    if let Some(local_transform) = world.get_local_transform_mut(camera) {
        local_transform.translation = position;
        local_transform.rotation = rotation;
    }

    if let Some(camera_component) = world.get_camera_mut(camera) {
        *camera_component = Camera {
            projection: Projection::Perspective(PerspectiveCamera {
                aspect_ratio: None,
                y_fov_rad: 45.0_f32.to_radians(),
                z_far: None,
                z_near: 0.01,
            }),
            smoothing: None,
        };
    }

    if let Some(pan_orbit_component) = world.get_pan_orbit_camera_mut(camera) {
        *pan_orbit_component = pan_orbit;
    }

    camera
}

pub fn spawn_ortho_camera(world: &mut World, position: Vec2) -> Entity {
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
        *camera_name = Name("Ortho Camera".to_string());
    }

    let ortho = OrthographicCamera {
        x_mag: 960.0,
        y_mag: 540.0,
        z_far: 1000.0,
        z_near: 0.01,
    };

    if let Some(local_transform) = world.get_local_transform_mut(camera) {
        local_transform.translation = Vec3::new(position.x, position.y, ortho.z_far);
    }

    if let Some(camera_component) = world.get_camera_mut(camera) {
        *camera_component = Camera {
            projection: Projection::Orthographic(ortho),
            smoothing: None,
        };
    }

    world.resources.active_camera = Some(camera);
    camera
}
