use crate::commands::{floor, slab};
use crate::resources::Sign;
use crate::theme;
use nalgebra_glm::vec3;
use nightshade::ecs::system_param::{Commands, ResMut};
use nightshade_camera::resources::ActiveCamera;
use nightshade_camera_controller::commands::spawn_pan_orbit_camera_at;
use nightshade_graphics::commands::builtins::spawn_sun;
use nightshade_render::resources::material_registry::MaterialRegistry;

pub fn stand_it_up(
    mut active_camera: ResMut<ActiveCamera>,
    mut material_registry: ResMut<MaterialRegistry>,
    mut sign: ResMut<Sign>,
    mut commands: Commands,
) {
    let camera = {
        let commands = &mut commands;

        spawn_pan_orbit_camera_at(
            commands,
            theme::CAMERA_TARGET,
            theme::CAMERA_BACK,
            theme::CAMERA_YAW,
            theme::CAMERA_PITCH,
            "Camera".to_string(),
        )
    };
    active_camera.0 = Some(camera);

    let post;
    {
        let commands = &mut commands;

        let materials = &mut *material_registry;
        floor(commands, materials, theme::REACH);
        slab(
            commands,
            materials,
            vec3(0.0, 1.2, 0.0),
            vec3(1.4, 2.4, 1.4),
            theme::BODY_TINT,
        );
        post = Some(slab(
            commands,
            materials,
            vec3(0.0, theme::SIGN_LOW * 0.5, theme::SIGN_BACK),
            vec3(0.8, theme::SIGN_LOW, 0.8),
            theme::SIGN_TINT,
        ));
        spawn_sun(commands);
    };
    sign.0 = post;
}
