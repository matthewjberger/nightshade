use crate::ecs::camera::components::{
    ThirdPersonCamera, compute_pan_orbit_transform, compute_third_person_transform,
};
use crate::ecs::world::{Entity, PAN_ORBIT_CAMERA, THIRD_PERSON_CAMERA, Vec2, Vec3, World};

use crate::prelude::*;
pub fn spawn_camera(world: &mut World, position: Vec3, name: String) -> Entity {
    let cameras = spawn_entities(
        world,
        crate::ecs::world::NAME
            | crate::ecs::world::LOCAL_TRANSFORM
            | crate::ecs::world::GLOBAL_TRANSFORM
            | crate::ecs::world::CAMERA,
        1,
    );

    let camera = cameras[0];

    if let Some(camera_name) = world.get_mut::<crate::ecs::primitives::Name>(camera) {
        *camera_name = Name(name);
    }

    if let Some(local_transform) =
        world.get_mut::<crate::ecs::transform::components::LocalTransform>(camera)
    {
        local_transform.translation = position;
    }

    if let Some(camera_component) = world.get_mut::<crate::ecs::camera::components::Camera>(camera)
    {
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

/// Spawns a pan-orbit camera with the default configuration, focused on the
/// origin. Reach for [`spawn_pan_orbit_camera_at`] or
/// [`spawn_pan_orbit_camera_with_config`] when you need to place or tune it.
pub fn spawn_pan_orbit_camera(world: &mut World) -> Entity {
    spawn_pan_orbit_camera_with_config(
        world,
        "Pan Orbit Camera".to_string(),
        PanOrbitCamera::default(),
    )
}

pub fn spawn_pan_orbit_camera_at(
    world: &mut World,
    focus: Vec3,
    radius: f32,
    yaw: f32,
    pitch: f32,
    name: String,
) -> Entity {
    let config = PanOrbitCamera::new(focus, radius).with_yaw_pitch(yaw, pitch);
    spawn_pan_orbit_camera_with_config(world, name, config)
}

pub fn spawn_pan_orbit_camera_with_config(
    world: &mut World,
    name: String,
    config: PanOrbitCamera,
) -> Entity {
    let cameras = spawn_entities(
        world,
        crate::ecs::world::NAME
            | crate::ecs::world::LOCAL_TRANSFORM
            | crate::ecs::world::GLOBAL_TRANSFORM
            | crate::ecs::world::CAMERA
            | PAN_ORBIT_CAMERA,
        1,
    );

    let camera = cameras[0];

    if let Some(camera_name) = world.get_mut::<crate::ecs::primitives::Name>(camera) {
        *camera_name = Name(name);
    }

    let (position, rotation) =
        compute_pan_orbit_transform(config.focus, config.yaw, config.pitch, config.radius);

    if let Some(local_transform) =
        world.get_mut::<crate::ecs::transform::components::LocalTransform>(camera)
    {
        local_transform.translation = position;
        local_transform.rotation = rotation;
    }

    if let Some(camera_component) = world.get_mut::<crate::ecs::camera::components::Camera>(camera)
    {
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

    if let Some(pan_orbit_component) =
        world.get_mut::<crate::ecs::camera::components::PanOrbitCamera>(camera)
    {
        *pan_orbit_component = config;
    }

    camera
}

pub fn spawn_ortho_camera(world: &mut World, position: Vec2) -> Entity {
    let cameras = spawn_entities(
        world,
        crate::ecs::world::NAME
            | crate::ecs::world::LOCAL_TRANSFORM
            | crate::ecs::world::GLOBAL_TRANSFORM
            | crate::ecs::world::CAMERA,
        1,
    );
    let camera = cameras[0];

    if let Some(camera_name) = world.get_mut::<crate::ecs::primitives::Name>(camera) {
        *camera_name = Name("Ortho Camera".to_string());
    }

    let ortho = OrthographicCamera {
        x_mag: 960.0,
        y_mag: 540.0,
        z_far: 1000.0,
        z_near: 0.01,
    };

    if let Some(local_transform) =
        world.get_mut::<crate::ecs::transform::components::LocalTransform>(camera)
    {
        local_transform.translation = Vec3::new(position.x, position.y, ortho.z_far);
    }

    if let Some(camera_component) = world.get_mut::<crate::ecs::camera::components::Camera>(camera)
    {
        *camera_component = Camera {
            projection: Projection::Orthographic(ortho),
            smoothing: None,
        };
    }

    world
        .res_mut::<crate::ecs::camera::resources::ActiveCamera>()
        .0 = Some(camera);
    camera
}

pub fn spawn_third_person_camera(
    world: &mut World,
    follow_target: Entity,
    distance: f32,
    yaw: f32,
    pitch: f32,
    name: String,
) -> Entity {
    let cameras = spawn_entities(
        world,
        crate::ecs::world::NAME
            | crate::ecs::world::LOCAL_TRANSFORM
            | crate::ecs::world::GLOBAL_TRANSFORM
            | crate::ecs::world::CAMERA
            | THIRD_PERSON_CAMERA,
        1,
    );

    let camera = cameras[0];

    if let Some(camera_name) = world.get_mut::<crate::ecs::primitives::Name>(camera) {
        *camera_name = Name(name);
    }

    let tpc = ThirdPersonCamera::new(distance)
        .with_target(follow_target)
        .with_yaw_pitch(yaw, pitch);

    let focus = world
        .get::<crate::ecs::transform::components::LocalTransform>(follow_target)
        .map(|t| t.translation + Vec3::new(0.0, tpc.height_offset, 0.0))
        .unwrap_or_default();

    let (position, rotation) = compute_third_person_transform(focus, yaw, pitch, distance);

    if let Some(local_transform) =
        world.get_mut::<crate::ecs::transform::components::LocalTransform>(camera)
    {
        local_transform.translation = position;
        local_transform.rotation = rotation;
    }

    if let Some(camera_component) = world.get_mut::<crate::ecs::camera::components::Camera>(camera)
    {
        *camera_component = Camera {
            projection: Projection::Perspective(PerspectiveCamera {
                aspect_ratio: None,
                y_fov_rad: 60.0_f32.to_radians(),
                z_far: Some(500.0),
                z_near: 0.1,
            }),
            smoothing: None,
        };
    }

    if let Some(tpc_component) =
        world.get_mut::<crate::ecs::camera::components::ThirdPersonCamera>(camera)
    {
        *tpc_component = tpc;
    }

    camera
}
