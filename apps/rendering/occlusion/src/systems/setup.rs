use crate::commands::dress;
use crate::theme;
use nightshade::prelude::*;
use nightshade_camera::prelude::ActiveCamera;
use nightshade_camera_controller::commands::spawn_pan_orbit_camera_at;
use nightshade_graphics::prelude::*;
use nightshade_render::contract::{Light, LightType};
use nightshade_render::data::material::Material;
use nightshade_render::queries::lights::sun_rotation;

pub fn initialize(
    mut commands: Commands,
    mut active_camera: ResMut<ActiveCamera>,
    mut render_settings: ResMut<RenderSettings>,
) {
    settle_the_look(&mut render_settings);
    let camera = {
        let commands = &mut commands;
        let camera = spawn_pan_orbit_camera_at(
            commands,
            theme::CAMERA_TARGET,
            theme::CAMERA_DISTANCE,
            theme::CAMERA_YAW,
            theme::CAMERA_PITCH,
            "Camera".to_string(),
        );
        commands.spawn((
            Name("Sun".to_string()),
            LocalTransform {
                translation: Vec3::new(0.0, 60.0, 0.0),
                rotation: sun_rotation(0.6, -0.85),
                scale: Vec3::new(1.0, 1.0, 1.0),
            },
            GlobalTransform::default(),
            Light {
                light_type: LightType::Directional,
                color: Vec3::new(1.0, 0.95, 0.86),
                intensity: 3.0,
                cast_shadows: true,
                shadow_bias: 0.02,
                ..Default::default()
            },
        ));
        let ground = spawn_mesh_at(
            commands,
            "Cube",
            Vec3::new(0.0, -0.5, 0.0),
            Vec3::new(260.0, 1.0, 260.0),
        );
        dress(
            commands,
            ground,
            Material {
                base_color: [0.20, 0.21, 0.24, 1.0],
                roughness: 0.72,
                metallic: 0.0,
                ..Default::default()
            },
        );

        let wall = spawn_mesh_at(
            commands,
            "Cube",
            Vec3::new(0.0, 7.0, -14.0),
            Vec3::new(theme::WALL, 14.0, 1.4),
        );
        dress(
            commands,
            wall,
            Material {
                base_color: [0.32, 0.30, 0.34, 1.0],
                roughness: 0.66,
                metallic: 0.0,
                ..Default::default()
            },
        );

        for index in 0..theme::HIDDEN {
            let across = ((index % theme::ACROSS) as f32 - (theme::ACROSS - 1) as f32 * 0.5) * 1.5;
            let back = ((index / theme::ACROSS) as f32) * 1.4 + 18.0;
            let lift = 0.5 + ((index % 6) as f32) * 1.1;
            let shape = spawn_mesh_at(
                commands,
                if index % 3 == 0 { "Sphere" } else { "Cube" },
                Vec3::new(across, lift, -back),
                Vec3::new(0.55, 0.55, 0.55),
            );
            dress(
                commands,
                shape,
                Material {
                    base_color: [0.80, 0.45, 0.22, 1.0],
                    roughness: 0.44,
                    metallic: 0.1,
                    ..Default::default()
                },
            );
        }

        camera
    };
    active_camera.0 = Some(camera);
}

fn settle_the_look(settings: &mut RenderSettings) {
    settings.atmosphere = Atmosphere::Sky;
    settings.bloom_enabled = true;
    settings.bloom_intensity = 0.5;
    settings.anti_aliasing = AntiAliasing::Taa;
    settings.ssao_enabled = true;
}
