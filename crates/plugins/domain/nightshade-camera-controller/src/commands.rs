use crate::contract::*;
use nalgebra_glm::Vec3;
use nightshade::prelude::*;
use nightshade_camera::components::{Camera, PerspectiveCamera, Projection};
use nightshade_scene::prelude::*;

fn perspective(y_fov_degrees: f32, z_near: f32, z_far: Option<f32>) -> Camera {
    Camera {
        projection: Projection::Perspective(PerspectiveCamera {
            aspect_ratio: None,
            y_fov_rad: y_fov_degrees.to_radians(),
            z_far,
            z_near,
        }),
        smoothing: None,
    }
}

pub fn spawn_pan_orbit_camera(commands: &mut Commands) -> Entity {
    spawn_pan_orbit_camera_with_config(
        commands,
        "Pan Orbit Camera".to_string(),
        PanOrbitCamera::default(),
    )
}

pub fn spawn_pan_orbit_camera_at(
    commands: &mut Commands,
    focus: Vec3,
    radius: f32,
    yaw: f32,
    pitch: f32,
    name: String,
) -> Entity {
    let config = PanOrbitCamera::new(focus, radius).with_yaw_pitch(yaw, pitch);
    spawn_pan_orbit_camera_with_config(commands, name, config)
}

pub fn spawn_pan_orbit_camera_with_config(
    commands: &mut Commands,
    name: String,
    config: PanOrbitCamera,
) -> Entity {
    let (position, rotation) =
        compute_pan_orbit_transform(config.focus, config.yaw, config.pitch, config.radius);
    let local_transform = nightshade_scene::contract::LocalTransform {
        translation: position,
        rotation,
        scale: Vec3::new(1.0, 1.0, 1.0),
    };

    commands.spawn((
        Name(name),
        local_transform,
        nightshade_scene::contract::GlobalTransform(local_transform.as_matrix()),
        perspective(45.0, 0.01, None),
        config,
    ))
}

pub fn spawn_third_person_camera(
    commands: &mut Commands,
    follow_target: Entity,
    distance: f32,
    yaw: f32,
    pitch: f32,
    name: String,
) -> Entity {
    let third_person = ThirdPersonCamera::new(distance)
        .with_target(follow_target)
        .with_yaw_pitch(yaw, pitch);
    let height_offset = third_person.height_offset;

    let camera = commands.spawn((
        Name(name),
        nightshade_scene::contract::LocalTransform::default(),
        nightshade_scene::contract::GlobalTransform::default(),
        perspective(60.0, 0.1, Some(500.0)),
        third_person,
    ));

    commands.queue(move |storage, _| {
        let focus = storage
            .get::<nightshade_scene::contract::LocalTransform>(follow_target)
            .map(|transform| transform.translation + Vec3::new(0.0, height_offset, 0.0))
            .unwrap_or_default();
        let (position, rotation) = compute_third_person_transform(focus, yaw, pitch, distance);
        if let Some(local_transform) =
            storage.get_mut::<nightshade_scene::contract::LocalTransform>(camera)
        {
            local_transform.translation = position;
            local_transform.rotation = rotation;
        }
    });

    camera
}

pub fn spawn_fly_camera_at(
    commands: &mut Commands,
    position: Vec3,
    yaw: f32,
    pitch: f32,
    name: String,
) -> Entity {
    let rotation = nalgebra_glm::quat_angle_axis(yaw, &Vec3::new(0.0, 1.0, 0.0))
        * nalgebra_glm::quat_angle_axis(pitch, &Vec3::new(1.0, 0.0, 0.0));
    let local_transform = nightshade_scene::contract::LocalTransform {
        translation: position,
        rotation,
        scale: Vec3::new(1.0, 1.0, 1.0),
    };
    commands.spawn((
        Name(name),
        local_transform,
        nightshade_scene::contract::GlobalTransform(local_transform.as_matrix()),
        perspective(45.0, 0.01, None),
    ))
}

pub fn frame_pan_orbit_camera(
    storage: &mut nightshade::ecs::Storage,
    camera: Entity,
    middle: Vec3,
    radius: f32,
) {
    if let Some(orbit) = storage.get_mut::<PanOrbitCamera>(camera) {
        orbit.focus = middle;
        orbit.target_focus = middle;
        orbit.radius = radius;
        orbit.target_radius = radius;
    }
}
