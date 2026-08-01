use crate::commands::{floor, slab};
use crate::resources::Run;
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
    mut run: ResMut<Run>,
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

    let mut runners = Vec::new();
    {
        let commands = &mut commands;

        let materials = &mut *material_registry;
        floor(commands, materials, theme::REACH);
        let start = -(theme::RUNNERS as f32 - 1.0) * theme::SPACING * 0.5;
        for step in 0..theme::RUNNERS {
            let along = start + step as f32 * theme::SPACING;
            runners.push(slab(
                commands,
                materials,
                vec3(theme::FROM, 0.5, along),
                vec3(1.0, 1.0, 1.0),
                theme::RUNNER_TINT,
            ));
            slab(
                commands,
                materials,
                vec3(theme::TO, 0.1, along),
                vec3(0.4, 0.2, 2.0),
                theme::MARK_TINT,
            );
        }
        spawn_sun(commands);
    };
    run.runners = runners;
}
