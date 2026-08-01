use crate::commands::{floor, slab};
use crate::resources::Scatter;
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
    mut scatter: ResMut<Scatter>,
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

    let mut discs = Vec::new();
    let mut rings = Vec::new();
    {
        let commands = &mut commands;

        let materials = &mut *material_registry;
        floor(commands, materials, theme::REACH);
        for _ in 0..theme::GRAINS {
            discs.push(slab(
                commands,
                materials,
                vec3(0.0, 0.4, 0.0),
                vec3(0.5, 0.5, 0.5),
                theme::DISC_TINT,
            ));
            rings.push(slab(
                commands,
                materials,
                vec3(0.0, 1.4, 0.0),
                vec3(0.4, 0.4, 0.4),
                theme::RING_TINT,
            ));
        }
        spawn_sun(commands);
    };
    scatter.discs = discs;
    scatter.rings = rings;
}
