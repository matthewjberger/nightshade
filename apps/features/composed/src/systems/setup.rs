use crate::commands::{ring_spot, spawn_grazer, spawn_slab};
use crate::data;
use crate::queries::meadow_height;
use crate::resources::Meadow;
use crate::theme;
use nalgebra_glm::vec3;
use nightshade::ecs::Storage;
use nightshade::ecs::system_param::{Commands, ResMut};
use nightshade_camera::commands::scene::spawn_camera;
use nightshade_camera::resources::ActiveCamera;
use nightshade_graphics::commands::builtins::spawn_sun;
use nightshade_ground::Ground;
use nightshade_render::resources::material_registry::MaterialRegistry;

pub fn stand_the_meadow(
    mut ground: ResMut<Ground>,
    mut active_camera: ResMut<ActiveCamera>,
    mut material_registry: ResMut<MaterialRegistry>,
    mut meadow: ResMut<Meadow>,
    mut commands: Commands,
    storage: &mut Storage,
) {
    ground.height_at = Some(meadow_height);

    let camera = {
        let commands = &mut commands;

        let camera = spawn_camera(commands, theme::CAMERA_BACK, "Meadow Camera".to_string());
        commands.queue(move |storage, _resources| {
            storage.set(camera, nightshade_shake::Trauma(0.0));
            storage.set(camera, nightshade_shake::ShakeOffset(vec3(0.0, 0.0, 0.0)));
            storage.set(camera, nightshade_shake::ShakeTurn(0.0));
            storage.set(camera, nightshade_follow::FollowOffset(theme::CAMERA_BACK));
        });
        camera
    };
    active_camera.0 = Some(camera);

    let mut watched = None;
    {
        let commands = &mut commands;

        let materials = &mut *material_registry;
        spawn_slab(
            commands,
            materials,
            data::MEADOW,
            vec3(0.0, -0.5, 0.0),
            vec3(theme::MEADOW_REACH * 2.0, 1.0, theme::MEADOW_REACH * 2.0),
            theme::GROUND_TINT,
        );
        for step in 0..8 {
            let spot = ring_spot(step, 8, theme::MEADOW_REACH * 0.8);
            spawn_slab(
                commands,
                materials,
                data::STONE,
                vec3(spot.x, 0.9, spot.z),
                vec3(1.4, 1.8, 1.4),
                theme::STONE_TINT,
            );
        }
        for step in 0..theme::GRAZERS {
            let home = ring_spot(step, theme::GRAZERS, theme::MEADOW_REACH * 0.45);
            let grazer = spawn_grazer(commands, materials, home);
            if watched.is_none() {
                watched = Some(grazer);
            }
        }
        spawn_sun(commands);
    };

    if let Some(watched) = watched {
        storage.set(camera, nightshade_follow::Following(watched));
    }
    meadow.camera = Some(camera);
    meadow.watched = watched;
}
