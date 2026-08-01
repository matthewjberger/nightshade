use crate::components::*;
use nalgebra_glm::{Vec2, Vec3};
use nightshade_ecs::Entity;
use nightshade_ecs::system_param::Commands;
use nightshade_scene::contract::primitives::Name;

pub fn spawn_camera(commands: &mut Commands, position: Vec3, name: String) -> Entity {
    commands.spawn((
        Name(name),
        nightshade_scene::contract::LocalTransform {
            translation: position,
            ..Default::default()
        },
        nightshade_scene::contract::GlobalTransform::default(),
        Camera {
            projection: Projection::Perspective(PerspectiveCamera {
                aspect_ratio: None,
                y_fov_rad: 45.0_f32.to_radians(),
                z_far: None,
                z_near: 0.01,
            }),
            smoothing: Some(Smoothing::default()),
        },
    ))
}

pub fn spawn_ortho_camera(commands: &mut Commands, position: Vec2) -> Entity {
    let ortho = OrthographicCamera {
        x_mag: None,
        y_mag: 540.0,
        z_far: 1000.0,
        z_near: 0.01,
    };

    let camera = commands.spawn((
        Name("Ortho Camera".to_string()),
        nightshade_scene::contract::LocalTransform {
            translation: Vec3::new(position.x, position.y, ortho.z_far),
            ..Default::default()
        },
        nightshade_scene::contract::GlobalTransform::default(),
        Camera {
            projection: Projection::Orthographic(ortho),
            smoothing: None,
        },
    ));

    commands.queue(move |_, resources| {
        resources.res_mut::<crate::resources::ActiveCamera>().0 = Some(camera);
    });
    camera
}

pub fn spawn_first_person_camera(
    commands: &mut Commands,
    parent: Entity,
    offset: Vec3,
    y_fov_degrees: f32,
) -> Entity {
    let camera = commands.spawn((
        Name("First Person Camera".to_string()),
        nightshade_scene::contract::LocalTransform {
            translation: offset,
            ..Default::default()
        },
        nightshade_scene::contract::GlobalTransform::default(),
        nightshade_scene::contract::ChildOf(parent),
        Camera {
            projection: Projection::Perspective(PerspectiveCamera {
                aspect_ratio: None,
                y_fov_rad: y_fov_degrees.to_radians(),
                z_far: None,
                z_near: 0.01,
            }),
            smoothing: Some(Smoothing {
                mouse_sensitivity: 1.5,
                ..Default::default()
            }),
        },
    ));
    commands.queue(move |_, resources| {
        resources.res_mut::<crate::resources::ActiveCamera>().0 = Some(camera);
    });
    camera
}

pub fn spawn_flat_camera(
    commands: &mut Commands,
    name: String,
    at: Vec3,
    y_mag: f32,
    z_near: f32,
    z_far: f32,
) -> Entity {
    commands.spawn((
        Name(name),
        nightshade_scene::contract::LocalTransform {
            translation: at,
            ..Default::default()
        },
        nightshade_scene::contract::GlobalTransform::default(),
        Camera {
            projection: Projection::Orthographic(OrthographicCamera {
                x_mag: None,
                y_mag,
                z_near,
                z_far,
            }),
            smoothing: None,
        },
    ))
}
