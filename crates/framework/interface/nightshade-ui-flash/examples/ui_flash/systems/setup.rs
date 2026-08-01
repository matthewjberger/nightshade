use crate::commands::{block, floor};
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
        floor(commands, materials);
        spawn_sun(commands);
        let start = -(theme::BLOCKS as f32 - 1.0) * theme::SPACING * 0.5;
        for step in 0..theme::BLOCKS {
            block(
                commands,
                materials,
                "Cube",
                vec3(start + step as f32 * theme::SPACING, 1.0, 0.0),
                vec3(1.6, 2.0, 1.6),
                theme::BLOCK_TINT,
            );
        }
    };
}
