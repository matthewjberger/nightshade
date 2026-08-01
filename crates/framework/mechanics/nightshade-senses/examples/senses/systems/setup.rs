use crate::commands::{block, floor};
use crate::components::Watcher;
use crate::resources::Field;
use crate::theme;
use nalgebra_glm::vec3;
use nightshade::ecs::Storage;
use nightshade::ecs::system_param::{Commands, ResMut};
use nightshade_camera::resources::ActiveCamera;
use nightshade_camera_controller::commands::spawn_pan_orbit_camera_at;
use nightshade_graphics::commands::builtins::spawn_sun;
use nightshade_render::resources::material_registry::MaterialRegistry;
use nightshade_senses::{Alert, Exposure, Hearing, LastSeen, Sight};

pub fn stand_it_up(
    mut active_camera: ResMut<ActiveCamera>,
    mut material_registry: ResMut<MaterialRegistry>,
    mut field: ResMut<Field>,
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

    let hider;
    let mut watchers = Vec::new();
    {
        let commands = &mut commands;

        let materials = &mut *material_registry;
        floor(commands, materials);
        spawn_sun(commands);
        let body = block(
            commands,
            materials,
            "Cube",
            vec3(0.0, 0.6, 8.0),
            vec3(0.9, 1.2, 0.9),
            theme::HIDER_TINT,
        );
        commands.queue(move |storage, _resources| {
            storage.set(body, Exposure(1.0));
        });
        hider = Some(body);
        for step in 0..theme::WATCHERS {
            let angle = step as f32 / theme::WATCHERS as f32 * std::f32::consts::TAU;
            let watcher = block(
                commands,
                materials,
                "Cube",
                vec3(
                    angle.cos() * theme::WATCHER_RING,
                    0.8,
                    angle.sin() * theme::WATCHER_RING,
                ),
                vec3(1.1, 1.6, 1.1),
                theme::WATCHER_CALM,
            );
            commands.queue(move |storage, _resources| {
                storage.set(watcher, Hearing(theme::HEARING));
                storage.set(watcher, Sight(theme::SIGHT));
                storage.set(watcher, Alert(0.0));
                storage.set(watcher, LastSeen(nalgebra_glm::Vec3::zeros()));
                storage.set(watcher, Watcher(step));
            });
            watchers.push(watcher);
        }
    };
    field.hider = hider;
    field.watchers = watchers;
}
