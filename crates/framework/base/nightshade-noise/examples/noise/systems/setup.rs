use crate::commands::{column, slab};
use crate::queries::{height_at, hill_height, tint_for};
use crate::resources::Field;
use crate::theme;
use nalgebra_glm::vec3;
use nightshade::ecs::Storage;
use nightshade::ecs::system_param::{Commands, ResMut};
use nightshade_camera::resources::ActiveCamera;
use nightshade_camera_controller::commands::spawn_pan_orbit_camera_at;
use nightshade_graphics::commands::builtins::spawn_sun;
use nightshade_ground::Ground;
use nightshade_ground::commands::answer_height_with;
use nightshade_grounded::Grounded;
use nightshade_render::resources::material_registry::MaterialRegistry;

pub fn raise_the_hills(
    mut ground: ResMut<Ground>,
    mut active_camera: ResMut<ActiveCamera>,
    mut material_registry: ResMut<MaterialRegistry>,
    mut field: ResMut<Field>,
    mut commands: Commands,
    _storage: &mut Storage,
) {
    answer_height_with(&mut ground, hill_height);

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

    let (seed, layers) = (theme::SEED, theme::LAYERS);

    let walker;
    let mut columns = Vec::new();
    {
        let commands = &mut commands;

        let materials = &mut *material_registry;
        let half = theme::COLUMNS / 2;
        for down in -half..half {
            for across in -half..half {
                let spot_across = across as f32 * theme::STEP;
                let spot_along = down as f32 * theme::STEP;
                let height = height_at(spot_across, spot_along, seed, layers);
                columns.push(column(
                    commands,
                    materials,
                    spot_across,
                    spot_along,
                    height,
                    tint_for(height),
                ));
            }
        }
        let standing = slab(
            commands,
            materials,
            vec3(0.0, 4.0, 0.0),
            vec3(0.8, 1.2, 0.8),
            theme::WALKER_TINT,
        );
        commands.queue(move |storage, _resources| {
            storage.set(standing, Grounded(theme::WALKER_LIFT));
            storage.set(standing, crate::components::Seed(theme::SEED));
            storage.set(standing, crate::components::Layers(theme::LAYERS));
        });
        walker = Some(standing);
        spawn_sun(commands);
    };

    field.walker = walker;
    field.columns = columns;
}
