use crate::commands::slab;
use crate::resources::Field;
use crate::theme;
use nalgebra_glm::vec3;
use nightshade::ecs::Storage;
use nightshade::ecs::system_param::{Commands, ResMut};
use nightshade_camera::resources::ActiveCamera;
use nightshade_camera_controller::commands::spawn_pan_orbit_camera_at;
use nightshade_graphics::commands::builtins::spawn_sun;
use nightshade_near::{Bulk, Crowded};
use nightshade_render::resources::material_registry::MaterialRegistry;

pub fn stand_it_up(
    mut active_camera: ResMut<ActiveCamera>,
    mut material_registry: ResMut<MaterialRegistry>,
    mut field: ResMut<Field>,
    mut commands: Commands,
    storage: &mut Storage,
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

    let mut crowd = Vec::new();
    let seeker = {
        let commands = &mut commands;

        let materials = &mut *material_registry;
        let reach = theme::COLUMNS as f32 * theme::SPACING * 0.5;
        slab(
            commands,
            materials,
            vec3(0.0, -0.6, 0.0),
            vec3(reach, 0.2, reach),
            theme::FLOOR_TINT,
        );
        let half = theme::COLUMNS / 2;
        for down in -half..half {
            for across in -half..half {
                if down == 0 && across == 0 {
                    continue;
                }
                crowd.push(slab(
                    commands,
                    materials,
                    vec3(
                        across as f32 * theme::SPACING,
                        0.0,
                        down as f32 * theme::SPACING,
                    ),
                    vec3(1.0, 1.0, 1.0),
                    theme::CROWD_TINT,
                ));
            }
        }
        let seeker = slab(
            commands,
            materials,
            vec3(0.0, 0.0, 0.0),
            vec3(1.4, 1.4, 1.4),
            theme::SEEKER_TINT,
        );
        spawn_sun(commands);
        seeker
    };
    for entity in crowd {
        storage.set(entity, Bulk(theme::BULK));
        storage.set(entity, Crowded(1.0));
    }
    storage.set(seeker, Bulk(theme::BULK));
    field.seeker = Some(seeker);
}
