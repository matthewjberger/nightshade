use crate::commands::floor;
use crate::theme;
use nightshade::ecs::system_param::{Commands, ResMut};
use nightshade_camera::resources::ActiveCamera;
use nightshade_camera_controller::commands::spawn_pan_orbit_camera_at;
use nightshade_graphics::commands::builtins::spawn_sun;
use nightshade_render::resources::material_registry::MaterialRegistry;
use nightshade_waves::{Wave, WavePlan};

pub fn open_the_gate(
    mut active_camera: ResMut<ActiveCamera>,
    mut wave_plan: ResMut<WavePlan>,
    mut material_registry: ResMut<MaterialRegistry>,
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

    wave_plan.0 = vec![
        Wave {
            count: 6,
            spacing: 0.35,
            rest: 4.0,
        },
        Wave {
            count: 10,
            spacing: 0.25,
            rest: 4.0,
        },
        Wave {
            count: 16,
            spacing: 0.18,
            rest: 5.0,
        },
    ];

    {
        let commands = &mut commands;

        let materials = &mut *material_registry;
        floor(commands, materials, theme::REACH);
        spawn_sun(commands);
    };
}
