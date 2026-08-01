use crate::commands::{floor, slab};
use crate::theme;
use nalgebra_glm::vec3;
use nightshade::ecs::Storage;
use nightshade::ecs::system_param::{Commands, ResMut};
use nightshade_camera::resources::ActiveCamera;
use nightshade_camera_controller::commands::spawn_pan_orbit_camera_at;
use nightshade_graphics::commands::builtins::spawn_sun;
use nightshade_render::resources::material_registry::MaterialRegistry;
use nightshade_topdown::{Heading, Walking};

pub fn stand_the_yard(
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
        floor(commands, materials, theme::REACH);
        for step in 0..6 {
            let turn = step as f32 / 6.0 * std::f32::consts::TAU;
            slab(
                commands,
                materials,
                vec3(turn.cos() * 9.0, 0.6, turn.sin() * 9.0),
                vec3(0.8, 1.2, 0.8),
                theme::POST_TINT,
            );
        }
        let walker = slab(
            commands,
            materials,
            vec3(0.0, 0.5, 0.0),
            vec3(1.0, 1.0, 1.0),
            theme::WALKER_TINT,
        );
        commands.queue(move |storage, _resources| {
            storage.set(walker, Walking(1.0));
            storage.set(walker, Heading(nalgebra_glm::vec2(0.0, 0.0)));
        });
        spawn_sun(commands);
    };
}
