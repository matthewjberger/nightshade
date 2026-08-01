use crate::commands::{Surface, dress, light_surface};
use crate::theme;
use nightshade::prelude::*;
use nightshade_camera::prelude::ActiveCamera;
use nightshade_camera_controller::commands::spawn_pan_orbit_camera_at;
use nightshade_graphics::prelude::*;
use nightshade_render::contract::AreaLightShape;
use nightshade_render::data::material::Material;

pub fn initialize(
    mut commands: Commands,
    mut render_settings: ResMut<RenderSettings>,
    mut active_camera: ResMut<ActiveCamera>,
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
            Vec3::new(120.0, 1.0, 120.0),
        );
        dress(
            commands,
            ground,
            Material {
                base_color: [0.30, 0.30, 0.33, 1.0],
                roughness: 0.35,
                metallic: 0.0,
                ..Default::default()
            },
        );

        for index in 0..theme::SPHERES {
            let along = (index as f32 - (theme::SPHERES as f32 - 1.0) * 0.5) * theme::SPHERE_PITCH;
            let sphere = spawn_mesh_at(
                commands,
                "Sphere",
                Vec3::new(along, theme::SPHERE_RADIUS, 0.0),
                Vec3::new(
                    theme::SPHERE_RADIUS,
                    theme::SPHERE_RADIUS,
                    theme::SPHERE_RADIUS,
                ),
            );
            dress(
                commands,
                sphere,
                Material {
                    base_color: [0.85, 0.85, 0.88, 1.0],
                    roughness: 0.05 + index as f32 / (theme::SPHERES as f32 - 1.0) * 0.85,
                    metallic: 1.0,
                    ..Default::default()
                },
            );
        }

        let facing_down = nalgebra_glm::quat_angle_axis(-std::f32::consts::FRAC_PI_2, &Vec3::x());
        for row in 0..theme::PANELS_ACROSS {
            for column in 0..theme::PANELS_ALONG {
                let index = row * theme::PANELS_ALONG + column;
                let tint = theme::TINTS[index % theme::TINTS.len()];
                let at = Vec3::new(
                    (column as f32 - (theme::PANELS_ALONG as f32 - 1.0) * 0.5) * theme::PANEL_PITCH,
                    theme::PANEL_HEIGHT,
                    (row as f32 - (theme::PANELS_ACROSS as f32 - 1.0) * 0.5) * theme::PANEL_PITCH
                        + theme::PANEL_PITCH * 2.0,
                );
                light_surface(
                    commands,
                    format!("Panel {index}"),
                    Surface {
                        shape: AreaLightShape::Rectangle,
                        at,
                        rotation: facing_down,
                        width: theme::PANEL_WIDTH,
                        height: theme::PANEL_DEPTH,
                        radius: 0.0,
                    },
                    tint,
                    theme::PANEL_INTENSITY,
                    theme::PANEL_RANGE,
                );
            }
        }

        light_surface(
            commands,
            "Disk".to_string(),
            Surface {
                shape: AreaLightShape::Disk,
                at: Vec3::new(-9.0, 4.5, -7.0),
                rotation: facing_down,
                width: 0.0,
                height: 0.0,
                radius: 1.4,
            },
            Vec3::new(1.0, 0.95, 0.85),
            10.0,
            16.0,
        );
        light_surface(
            commands,
            "Sphere".to_string(),
            Surface {
                shape: AreaLightShape::Sphere,
                at: Vec3::new(0.0, 4.0, -7.0),
                rotation: Quat::identity(),
                width: 0.0,
                height: 0.0,
                radius: 0.8,
            },
            Vec3::new(0.85, 0.95, 1.0),
            13.0,
            16.0,
        );
        light_surface(
            commands,
            "Tube".to_string(),
            Surface {
                shape: AreaLightShape::Tube,
                at: Vec3::new(9.0, 3.5, -7.0),
                rotation: Quat::identity(),
                width: 6.0,
                height: 0.0,
                radius: 0.18,
            },
            Vec3::new(1.0, 0.75, 0.55),
            15.0,
            16.0,
        );

        camera
    };
    active_camera.0 = Some(camera);
}

fn settle_the_look(settings: &mut RenderSettings) {
    settings.atmosphere = Atmosphere::None;
    settings.bloom_enabled = true;
    settings.bloom_intensity = 0.6;
    settings.anti_aliasing = AntiAliasing::Taa;
    settings.bloom_threshold = 1.0;
}
