use crate::theme;
use nalgebra_glm::Vec3;
use nightshade::prelude::*;
use nightshade_camera::prelude::*;
use nightshade_camera::queries::camera_look::camera_look_rotation;
use nightshade_scene::prelude::*;

pub fn spawn_camera(commands: &mut Commands, looked_at: Vec3) -> Entity {
    let eye = looked_at + Vec3::new(0.0, 9.0, 12.0);
    commands.spawn((
        Name("Battle Camera".to_string()),
        LocalTransform {
            translation: eye,
            rotation: camera_look_rotation(eye, looked_at, 0.0),
            ..Default::default()
        },
        GlobalTransform::default(),
        Camera {
            projection: Projection::Perspective(PerspectiveCamera {
                aspect_ratio: None,
                y_fov_rad: theme::CAMERA_FOV_DEGREES.to_radians(),
                z_far: Some(theme::CAMERA_FAR),
                z_near: theme::CAMERA_NEAR,
            }),
            smoothing: None,
        },
    ))
}
