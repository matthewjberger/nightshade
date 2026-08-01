use crate::commands::{block, floor, torch};
use crate::resources::Torch;
use crate::theme;
use nalgebra_glm::vec3;
use nightshade::ecs::Storage;
use nightshade::ecs::system_param::{Commands, ResMut};
use nightshade_camera::resources::ActiveCamera;
use nightshade_camera_controller::commands::spawn_pan_orbit_camera_at;
use nightshade_flashlight::{Battery, Beam, Drain, Lit};
use nightshade_render::resources::material_registry::MaterialRegistry;

pub fn stand_it_up(
    mut active_camera: ResMut<ActiveCamera>,
    mut material_registry: ResMut<MaterialRegistry>,
    mut torch_held: ResMut<Torch>,
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

    let carried;
    {
        let commands = &mut commands;

        let materials = &mut *material_registry;
        floor(commands, materials);
        for step in 0..theme::CRATES {
            let angle = step as f32 / theme::CRATES as f32 * std::f32::consts::TAU;
            block(
                commands,
                materials,
                "Cube",
                vec3(angle.cos() * theme::RING, 0.6, angle.sin() * theme::RING),
                vec3(1.2, 1.2, 1.2),
                theme::CRATE_TINT,
            );
        }
        let light = torch(commands);
        commands.queue(move |storage, _resources| {
            storage.set(light, Battery(theme::FULL));
            storage.set(light, Drain(theme::DRAIN));
            storage.set(light, Lit(true));
            storage.set(light, Beam(theme::BEAM));
        });
        carried = Some(light);
    };
    torch_held.entity = carried;
}
