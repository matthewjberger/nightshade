use crate::commands::{floor, slab};
use crate::resources::Walk;
use crate::theme;
use nalgebra_glm::vec3;
use nightshade::ecs::system_param::{Commands, ResMut};
use nightshade_camera::resources::ActiveCamera;
use nightshade_camera_controller::commands::spawn_pan_orbit_camera_at;
use nightshade_graphics::commands::builtins::spawn_sun;
use nightshade_render::resources::material_registry::MaterialRegistry;

pub fn stand_it_up(
    mut active_camera: ResMut<ActiveCamera>,
    mut material_registry: ResMut<MaterialRegistry>,
    mut walk: ResMut<Walk>,
    mut commands: Commands,
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

    let walker;
    {
        let commands = &mut commands;

        let materials = &mut *material_registry;
        floor(commands, materials, theme::REACH);
        slab(
            commands,
            materials,
            vec3(0.0, 0.2, 0.0),
            vec3(0.6, 0.4, 0.6),
            theme::MIDDLE_TINT,
        );
        walker = Some(slab(
            commands,
            materials,
            vec3(theme::RING * 0.5, 0.6, 0.0),
            vec3(1.2, 1.2, 1.2),
            theme::GROUND_TINT,
        ));
        spawn_sun(commands);
    };
    walk.walker = walker;
}
