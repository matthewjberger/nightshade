use crate::commands::{floor, slab};
use crate::resources::Swing;
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
    mut swing: ResMut<Swing>,
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

    let swinger;
    let mut targets = Vec::new();
    {
        let commands = &mut commands;

        let materials = &mut *material_registry;
        floor(commands, materials, theme::REACH);
        swinger = Some(slab(
            commands,
            materials,
            vec3(0.0, 1.0, 0.0),
            vec3(1.0, 2.0, 1.0),
            theme::SWINGER_TINT,
        ));
        for step in 0..theme::TARGETS {
            let angle = step as f32 / theme::TARGETS as f32 * std::f32::consts::TAU;
            targets.push(slab(
                commands,
                materials,
                vec3(angle.cos() * theme::RING, 0.5, angle.sin() * theme::RING),
                vec3(0.8, 1.0, 0.8),
                theme::TARGET_TINT,
            ));
        }
        spawn_sun(commands);
    };
    swing.swinger = swinger;
    swing.targets = targets;
}
