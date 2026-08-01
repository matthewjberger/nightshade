use crate::commands::{block, floor, lamp};
use crate::theme;
use nalgebra_glm::vec3;
use nightshade::ecs::Storage;
use nightshade::ecs::system_param::{Commands, ResMut};
use nightshade_camera::resources::ActiveCamera;
use nightshade_camera_controller::commands::spawn_pan_orbit_camera_at;
use nightshade_phase::PhaseTinted;
use nightshade_render::resources::material_registry::MaterialRegistry;

pub fn stand_it_up(
    mut active_camera: ResMut<ActiveCamera>,
    mut material_registry: ResMut<MaterialRegistry>,
    mut commands: Commands,
    _storage: &mut Storage,
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
        let start = -(theme::WALLS as f32 - 1.0) * theme::SPACING * 0.5;
        for step in 0..theme::WALLS {
            let across = start + step as f32 * theme::SPACING;
            for side in [-1.0_f32, 1.0] {
                block(
                    commands,
                    materials,
                    "Cube",
                    vec3(across, 2.0, side * 6.0),
                    vec3(3.6, 4.0, 0.6),
                    theme::WALL_TINT,
                );
            }
            if step % 2 == 0 {
                let light = lamp(commands, vec3(across, theme::LAMP_HEIGHT, 0.0));
                commands.queue(move |storage, _resources| {
                    storage.set(light, PhaseTinted(1.0));
                });
            }
        }
    };
}
