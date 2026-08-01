use crate::commands::{block, floor};
use crate::resources::Stage;
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
    mut stage: ResMut<Stage>,
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

    let ball;
    {
        let commands = &mut commands;

        let materials = &mut *material_registry;
        floor(commands, materials);
        let start = -(theme::PILLARS as f32 - 1.0) * theme::SPACING * 0.5;
        for step in 0..theme::PILLARS {
            let across = start + step as f32 * theme::SPACING;
            block(
                commands,
                materials,
                "Cube",
                vec3(across, theme::PILLAR_HEIGHT * 0.5, -4.0),
                vec3(1.2, theme::PILLAR_HEIGHT, 1.2),
                theme::PILLAR_TINT,
            );
        }
        ball = Some(block(
            commands,
            materials,
            "Sphere",
            vec3(0.0, 1.8, 2.0),
            vec3(1.8, 1.8, 1.8),
            theme::BALL_TINT,
        ));
        spawn_sun(commands);
    };
    stage.ball = ball;
    stage.wearing = "playstation";
}
