use crate::commands::{floor, slab};
use crate::resources::Sky;
use crate::theme;
use nalgebra_glm::vec3;
use nightshade::ecs::system_param::{Commands, Res, ResMut};
use nightshade_camera::resources::ActiveCamera;
use nightshade_camera_controller::commands::spawn_pan_orbit_camera_at;
use nightshade_daynight::DaySettings;
use nightshade_graphics::commands::builtins::spawn_sun;
use nightshade_render::resources::material_registry::MaterialRegistry;

pub fn stand_it_up(
    mut active_camera: ResMut<ActiveCamera>,
    mut material_registry: ResMut<MaterialRegistry>,
    day_settings: Res<DaySettings>,
    mut sky: ResMut<Sky>,
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

    let sun = {
        let commands = &mut commands;

        let materials = &mut *material_registry;
        floor(commands, materials, theme::REACH);
        for step in 0..theme::POSTS {
            let angle = step as f32 / theme::POSTS as f32 * std::f32::consts::TAU;
            slab(
                commands,
                materials,
                vec3(angle.cos() * theme::RING, 1.4, angle.sin() * theme::RING),
                vec3(0.8, 2.8, 0.8),
                theme::POST_TINT,
            );
        }
        spawn_sun(commands)
    };
    let held = day_settings.hours_a_second;
    sky.sun = Some(sun);
    sky.held = held;
}
