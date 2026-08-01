use crate::commands::dress;
use crate::theme;
use nightshade::prelude::*;
use nightshade_camera::prelude::ActiveCamera;
use nightshade_camera_controller::commands::spawn_pan_orbit_camera_at;
use nightshade_graphics::prelude::*;
use nightshade_render::contract::{Light, LightType};
use nightshade_render::data::material::Material;

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
        let ground = spawn_mesh_at(
            commands,
            "Cube",
            Vec3::new(0.0, -0.5, 0.0),
            Vec3::new(150.0, 1.0, 150.0),
        );
        dress(
            commands,
            ground,
            Material {
                base_color: [0.12, 0.12, 0.15, 1.0],
                roughness: 0.72,
                metallic: 0.0,
                ..Default::default()
            },
        );

        for index in 0..theme::BLOCKS {
            let along = index as f32 / theme::BLOCKS as f32;
            let turn = along * std::f32::consts::TAU * 5.0;
            let out = 6.0 + along * theme::SPREAD;
            let height = 1.0 + ((index % 5) as f32) * 1.6;
            let block = spawn_mesh_at(
                commands,
                "Cube",
                Vec3::new(out * turn.cos(), height * 0.5, out * turn.sin()),
                Vec3::new(2.4, height, 2.4),
            );
            dress(
                commands,
                block,
                Material {
                    base_color: [0.22, 0.22, 0.26, 1.0],
                    roughness: 0.48,
                    metallic: 0.0,
                    ..Default::default()
                },
            );
        }

        for index in 0..theme::LIGHTS {
            let along = index as f32 / theme::LIGHTS as f32;
            let turn = along * std::f32::consts::TAU * 9.0;
            let out = 4.0 + along * theme::SPREAD;
            let tint = theme::TINTS[index % theme::TINTS.len()];
            let at = Vec3::new(
                out * turn.cos(),
                2.2 + (turn * 3.0).sin() * 1.4,
                out * turn.sin(),
            );
            commands.spawn((
                Name(format!("Light {index}")),
                LocalTransform {
                    translation: at,
                    rotation: Quat::identity(),
                    scale: Vec3::new(1.0, 1.0, 1.0),
                },
                GlobalTransform::default(),
                Light {
                    light_type: LightType::Point,
                    color: tint,
                    intensity: 3.2,
                    range: 7.5,
                    cast_shadows: false,
                    ..Default::default()
                },
            ));
            let bulb = spawn_mesh_at(commands, "Sphere", at, Vec3::new(0.16, 0.16, 0.16));
            dress(
                commands,
                bulb,
                Material {
                    base_color: [0.0, 0.0, 0.0, 1.0],
                    emissive_factor: [tint.x * 4.0, tint.y * 4.0, tint.z * 4.0],
                    unlit: true,
                    ..Default::default()
                },
            );
        }

        for index in 0..theme::SHADOW_LIGHTS {
            let turn = index as f32 / theme::SHADOW_LIGHTS as f32 * std::f32::consts::TAU;
            let tint = theme::TINTS[index % theme::TINTS.len()];
            let at = Vec3::new(
                theme::SHADOW_RING * turn.cos(),
                theme::SHADOW_HEIGHT,
                theme::SHADOW_RING * turn.sin(),
            );
            commands.spawn((
                Name(format!("Shadow light {index}")),
                LocalTransform {
                    translation: at,
                    rotation: Quat::identity(),
                    scale: Vec3::new(1.0, 1.0, 1.0),
                },
                GlobalTransform::default(),
                Light {
                    light_type: LightType::Point,
                    color: tint,
                    intensity: 9.0,
                    range: theme::SHADOW_RANGE,
                    cast_shadows: true,
                    shadow_bias: 0.004,
                    ..Default::default()
                },
            ));
            let bulb = spawn_mesh_at(commands, "Sphere", at, Vec3::new(0.3, 0.3, 0.3));
            dress(
                commands,
                bulb,
                Material {
                    base_color: [0.0, 0.0, 0.0, 1.0],
                    emissive_factor: [tint.x * 4.0, tint.y * 4.0, tint.z * 4.0],
                    unlit: true,
                    ..Default::default()
                },
            );
        }

        camera
    };
    active_camera.0 = Some(camera);
}

fn settle_the_look(settings: &mut RenderSettings) {
    settings.atmosphere = Atmosphere::None;
    settings.bloom_enabled = true;
    settings.bloom_intensity = 1.1;
    settings.anti_aliasing = AntiAliasing::Taa;
    settings.bloom_threshold = 0.8;
}
