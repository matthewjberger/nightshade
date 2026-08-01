use crate::commands::{floor, slab};
use crate::theme;
use nalgebra_glm::vec3;
use nightshade::ecs::Storage;
use nightshade::ecs::system_param::{Commands, ResMut};
use nightshade_camera::resources::ActiveCamera;
use nightshade_camera_controller::commands::spawn_pan_orbit_camera_at;
use nightshade_graphics::commands::builtins::spawn_sun;
use nightshade_render::resources::material_registry::MaterialRegistry;
use nightshade_wander::{Home, Resting, Roaming, Wandering};

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
        floor(commands, materials, theme::REACH);
        let side = 3;
        for step in 0..theme::WALKERS {
            let across = (step % side) as f32 - 1.0;
            let along = (step / side) as f32 - 1.0;
            let home = vec3(
                across * theme::SPREAD * 0.5,
                0.6,
                along * theme::SPREAD * 0.5,
            );
            slab(
                commands,
                materials,
                vec3(home.x, 0.1, home.z),
                vec3(1.6, 0.2, 1.6),
                theme::HOME_TINT,
            );
            let walker = slab(
                commands,
                materials,
                home,
                vec3(0.9, 1.2, 0.9),
                theme::WALKER_TINT,
            );
            commands.queue(move |storage, _resources| {
                storage.set(walker, Home(home));
                storage.set(walker, Wandering(home));
                storage.set(walker, Resting(0.0));
                storage.set(walker, Roaming(1.0));
            });
        }
        spawn_sun(commands);
    };
}
