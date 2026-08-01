use crate::commands::slab;
use crate::queries::{height_of, hill_height, on_the_field};
use crate::resources::Walker;
use crate::theme;
use nalgebra_glm::vec3;
use nightshade::ecs::Storage;
use nightshade::ecs::system_param::{Commands, ResMut};
use nightshade_camera::resources::ActiveCamera;
use nightshade_camera_controller::commands::spawn_pan_orbit_camera_at;
use nightshade_graphics::commands::builtins::spawn_sun;
use nightshade_ground::Ground;
use nightshade_grounded::Grounded;
use nightshade_render::resources::material_registry::MaterialRegistry;

pub fn stand_it_up(
    mut ground: ResMut<Ground>,
    mut active_camera: ResMut<ActiveCamera>,
    mut material_registry: ResMut<MaterialRegistry>,
    mut walker_held: ResMut<Walker>,
    mut commands: Commands,
    _storage: &mut Storage,
) {
    ground.height_at = Some(hill_height);
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

    let walker = {
        let commands = &mut commands;

        let materials = &mut *material_registry;
        let half = theme::COLUMNS / 2;
        for down in -half..half {
            for across in -half..half {
                let spot_across = across as f32 * theme::STEP;
                let spot_along = down as f32 * theme::STEP;
                if !on_the_field(spot_across, spot_along) {
                    continue;
                }
                let high = height_of(spot_across, spot_along);
                slab(
                    commands,
                    materials,
                    vec3(spot_across, high * 0.5, spot_along),
                    vec3(theme::STEP, high.max(0.2), theme::STEP),
                    theme::HILL_TINT,
                );
            }
        }
        let walker = slab(
            commands,
            materials,
            vec3(0.0, 4.0, 0.0),
            vec3(0.8, 1.2, 0.8),
            theme::WALKER_TINT,
        );
        commands.queue(move |storage, _resources| {
            storage.set(walker, Grounded(theme::STAND));
        });
        spawn_sun(commands);
        walker
    };
    walker_held.body = Some(walker);
}
