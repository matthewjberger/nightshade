use crate::commands::{effects, interface, textures};
use crate::resources::{Interface, Launcher, Toggles};
use nightshade::prelude::*;
use nightshade_camera::prelude::*;
use nightshade_camera_controller::contract::*;
use nightshade_graphics::prelude::*;
use nightshade_particles::contract::ParticleTextures;
use nightshade_text::prelude::*;
use nightshade_ui::prelude::*;

pub fn initialize(
    launcher: Res<Launcher>,
    toggles: Res<Toggles>,
    mut panel: ResMut<Interface>,
    mut render_settings: ResMut<RenderSettings>,
    mut debug_draw: ResMut<DebugDraw>,
    mut active_camera: ResMut<ActiveCamera>,
    mut particle_textures: ResMut<ParticleTextures>,
    mut build: UiBuild,
) {
    render_settings.atmosphere = Atmosphere::Space;
    render_settings.clear_color = [0.01, 0.01, 0.02, 1.0];
    render_settings.bloom_enabled = true;
    render_settings.bloom_intensity = 0.05;
    debug_draw.show_grid = toggles.grid;

    textures::load_particle_textures(&mut particle_textures);

    let bloom_intensity = render_settings.bloom_intensity;
    let camera = {
        let context = build.context();
        let commands = context.commands;

        let camera = commands.spawn(());
        commands.set(
            camera,
            LocalTransform {
                translation: Vec3::new(0.0, 50.0, 150.0),
                rotation: Quat::identity(),
                scale: Vec3::new(1.0, 1.0, 1.0),
            },
        );
        commands.set(camera, GlobalTransform::default());
        commands.set(
            camera,
            Camera {
                projection: Projection::Perspective(PerspectiveCamera {
                    aspect_ratio: None,
                    y_fov_rad: 60.0_f32.to_radians(),
                    z_near: 0.1,
                    z_far: Some(2000.0),
                }),
                smoothing: Some(Smoothing::default()),
            },
        );
        commands.set(
            camera,
            PanOrbitCamera {
                focus: Vec3::new(0.0, 40.0, 0.0),
                target_focus: Vec3::new(0.0, 40.0, 0.0),
                radius: 150.0,
                target_radius: 150.0,
                pitch: 0.2,
                target_pitch: 0.2,
                yaw: 0.0,
                target_yaw: 0.0,
                ..Default::default()
            },
        );

        if toggles.fire {
            effects::spawn_fire_effect(commands, Vec3::new(-35.0, 0.0, 0.0));
            effects::spawn_fire_effect(commands, Vec3::new(35.0, 0.0, 0.0));
            effects::spawn_fire_effect(commands, Vec3::new(-25.0, 0.0, 10.0));
            effects::spawn_fire_effect(commands, Vec3::new(25.0, 0.0, 10.0));
        }

        if toggles.smoke {
            effects::spawn_smoke_effect(commands, Vec3::new(-30.0, 3.0, 5.0));
            effects::spawn_smoke_effect(commands, Vec3::new(30.0, 3.0, 5.0));
        }

        effects::spawn_all_sprite_effects(context.text_state, commands);

        spawn_ui_text_with_properties(
            context.text_state,
            commands,
            "GPU Particle System - Fireworks",
            Vec2::zeros(),
            TextProperties {
                font_size: 18.0,
                color: vec4(1.0, 1.0, 1.0, 0.9),
                alignment: TextAlignment::Center,
                outline_width: 0.01,
                outline_color: vec4(0.0, 0.0, 0.0, 1.0),
                ..Default::default()
            },
        );
        camera
    };
    active_camera.0 = Some(camera);

    interface::build_ui(&mut panel, &launcher, &toggles, &mut build, bloom_intensity);
}
