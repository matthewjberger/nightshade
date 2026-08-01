use crate::commands::{block, floor, lamp};
use crate::resources::Street;
use crate::theme;
use nalgebra_glm::vec3;
use nightshade::ecs::Storage;
use nightshade::ecs::system_param::{Commands, ResMut};
use nightshade_camera::resources::ActiveCamera;
use nightshade_camera_controller::commands::spawn_pan_orbit_camera_at;
use nightshade_flicker::{FlickerBase, Flickering};
use nightshade_render::resources::material_registry::MaterialRegistry;

pub fn stand_it_up(
    mut active_camera: ResMut<ActiveCamera>,
    mut material_registry: ResMut<MaterialRegistry>,
    mut street: ResMut<Street>,
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

    let mut lamps = Vec::new();
    {
        let commands = &mut commands;

        let materials = &mut *material_registry;
        floor(commands, materials);
        let start = -(theme::LAMPS as f32 - 1.0) * theme::SPACING * 0.5;
        for step in 0..theme::LAMPS {
            let across = start + step as f32 * theme::SPACING;
            block(
                commands,
                materials,
                "Cube",
                vec3(across, theme::LAMP_HEIGHT * 0.5, 0.0),
                vec3(0.3, theme::LAMP_HEIGHT, 0.3),
                theme::POST_TINT,
            );
            let brightness = theme::DIMMEST
                + (theme::BRIGHTEST - theme::DIMMEST) * step as f32 / (theme::LAMPS as f32 - 1.0);
            let light = lamp(commands, vec3(across, theme::LAMP_HEIGHT, 0.0), brightness);
            commands.queue(move |storage, _resources| {
                storage.set(light, FlickerBase(brightness));
                storage.set(light, Flickering(1.0));
            });
            lamps.push(light);
        }
    };
    street.lamps = lamps;
}
