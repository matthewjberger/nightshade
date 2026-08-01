use crate::theme;
use nightshade::prelude::*;
use nightshade_camera::prelude::ActiveCamera;
use nightshade_camera_controller::commands::spawn_pan_orbit_camera_at;
use nightshade_graphics::prelude::*;

pub fn initialize(
    mut commands: Commands,
    mut active_camera: ResMut<ActiveCamera>,
    mut command_queues: ResMut<CommandQueues>,
    mut render_settings: ResMut<RenderSettings>,
    mut materials: ResMut<MaterialRegistry>,
) {
    settle_the_look(&mut command_queues, &mut render_settings);
    let camera = {
        let commands = &mut commands;
        spawn_sun(commands);
        build_the_scene(commands, &mut materials);
        spawn_pan_orbit_camera_at(
            commands,
            Vec3::from(theme::CAMERA_FOCUS),
            theme::CAMERA_BACK,
            theme::CAMERA_YAW,
            theme::CAMERA_PITCH,
            "Camera".to_string(),
        )
    };
    active_camera.0 = Some(camera);
}

fn settle_the_look(command_queues: &mut CommandQueues, settings: &mut RenderSettings) {
    load_hdr_skybox(&mut *command_queues, theme::SKY.to_vec());
    settings.atmosphere = Atmosphere::Hdr;
    settings.bloom_enabled = true;
    settings.bloom_intensity = 0.3;
    settings.anti_aliasing = AntiAliasing::Taa;
    settings.ssr_enabled = true;
    settings.ssr_intensity = theme::REFLECTION_STRENGTH;
    settings.ssr_max_distance = theme::REFLECTION_REACH;
}

fn build_the_scene(
    commands: &mut nightshade::ecs::system_param::Commands<'_>,
    materials: &mut MaterialRegistry,
) {
    let floor = spawn_mesh_at(
        commands,
        "Cube",
        Vec3::new(0.0, -theme::FLOOR_THICKNESS, 0.0),
        Vec3::new(
            theme::FLOOR_REACH,
            theme::FLOOR_THICKNESS,
            theme::FLOOR_REACH,
        ),
    );
    dress(
        commands,
        materials,
        floor,
        Material {
            base_color: theme::FLOOR_COLOR,
            roughness: theme::FLOOR_ROUGHNESS,
            metallic: 1.0,
            ..Default::default()
        },
    );

    for (along, color, roughness, metallic) in theme::BALLS {
        let ball = spawn_mesh_at(
            commands,
            "Sphere",
            Vec3::new(along, theme::BALL_HEIGHT, theme::BALL_ROW),
            Vec3::new(theme::BALL_REACH, theme::BALL_REACH, theme::BALL_REACH),
        );
        dress(
            commands,
            materials,
            ball,
            Material {
                base_color: color,
                roughness,
                metallic,
                ..Default::default()
            },
        );
    }

    for (along, height, light) in theme::LAMPS {
        let lamp = spawn_mesh_at(
            commands,
            "Sphere",
            Vec3::new(along, height, theme::LAMP_ROW),
            Vec3::new(theme::LAMP_REACH, theme::LAMP_REACH, theme::LAMP_REACH),
        );
        dress(
            commands,
            materials,
            lamp,
            Material {
                base_color: [0.0, 0.0, 0.0, 1.0],
                emissive_factor: light,
                unlit: true,
                ..Default::default()
            },
        );
    }
}

fn dress(
    commands: &mut nightshade::ecs::system_param::Commands<'_>,
    materials: &mut MaterialRegistry,
    entity: nightshade::ecs::Entity,
    material: Material,
) {
    set_material(
        materials,
        commands,
        entity,
        format!("{} {}", theme::TITLE, entity.id),
        material,
    );
}
