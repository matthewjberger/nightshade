use crate::commands::{floor, slab};
use crate::resources::Row;
use crate::theme;
use nalgebra_glm::vec3;
use nightshade::ecs::Storage;
use nightshade::ecs::system_param::{Commands, ResMut};
use nightshade_camera::resources::ActiveCamera;
use nightshade_camera_controller::commands::spawn_pan_orbit_camera_at;
use nightshade_graphics::commands::builtins::spawn_sun;
use nightshade_hover::{Bob, Perch, Phase, Spin};
use nightshade_render::resources::material_registry::MaterialRegistry;

pub fn stand_it_up(
    mut active_camera: ResMut<ActiveCamera>,
    mut material_registry: ResMut<MaterialRegistry>,
    mut row: ResMut<Row>,
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

    let coins = {
        let commands = &mut commands;

        let materials = &mut *material_registry;
        floor(commands, materials, theme::REACH);
        let mut coins = Vec::new();
        let start = -(theme::COINS as f32 - 1.0) * theme::SPACING * 0.5;
        for step in 0..theme::COINS {
            let perch = vec3(start + step as f32 * theme::SPACING, theme::PERCH, 0.0);
            let coin = slab(
                commands,
                materials,
                perch,
                vec3(0.9, 0.9, 0.2),
                theme::COIN_TINT,
            );
            let bob = 0.1 + step as f32 * 0.12;
            let phase = step as f32 * theme::PHASE;
            commands.queue(move |storage, _resources| {
                storage.set(coin, Perch(perch));
                storage.set(coin, Bob(bob));
                storage.set(coin, Phase(phase));
                storage.set(coin, Spin(theme::SPIN));
            });
            coins.push(coin);
        }
        spawn_sun(commands);
        coins
    };
    row.coins = coins;
}
