use crate::commands::{floor, slab};
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

    {
        let commands = &mut commands;

        let materials = &mut *material_registry;
        floor(commands, materials, theme::REACH);
        for step in 0..theme::CLOUDS {
            let angle = step as f32 / theme::CLOUDS as f32 * std::f32::consts::TAU;
            slab(
                commands,
                materials,
                vec3(
                    angle.cos() * theme::SPREAD,
                    theme::CEILING,
                    angle.sin() * theme::SPREAD,
                ),
                vec3(3.0, 0.6, 3.0),
                theme::CLOUD_TINT,
            );
        }
        spawn_sun(commands);
    };
}
