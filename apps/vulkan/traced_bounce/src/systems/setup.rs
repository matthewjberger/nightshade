use crate::queries;
use crate::theme;
use nightshade::prelude::*;
use nightshade_camera::prelude::ActiveCamera;
use nightshade_camera_controller::commands::spawn_pan_orbit_camera_at;
use nightshade_graphics::prelude::*;
use nightshade_render::contract::{Light, LightType};

pub fn initialize(
    mut commands: Commands,
    mut active_camera: ResMut<ActiveCamera>,
    mut render_settings: ResMut<RenderSettings>,
    mut materials: ResMut<MaterialRegistry>,
) {
    settle_the_look(&mut render_settings);
    let camera = {
        let commands = &mut commands;
        build_the_box(commands, &mut materials);
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

fn settle_the_look(settings: &mut RenderSettings) {
    settings.show_sky = false;
    settings.clear_color = theme::BACKDROP;
    settings.bloom_enabled = true;
    settings.bloom_intensity = 0.15;
    settings.anti_aliasing = AntiAliasing::Taa;
    settings.ssgi_enabled = false;
    settings.color_grading.exposure = theme::EXPOSURE;
    settings.probe_volume = Some(queries::probe_volume());
}

fn build_the_box(
    commands: &mut nightshade::ecs::system_param::Commands<'_>,
    materials: &mut MaterialRegistry,
) {
    let reach = theme::REACH;
    let thickness = theme::THICKNESS;
    let side = reach * 2.0;
    let half_wall = thickness * 0.5;

    let surfaces = [
        (
            Vec3::new(0.0, -half_wall, 0.0),
            Vec3::new(side, thickness, side),
            theme::WHITE,
        ),
        (
            Vec3::new(0.0, side + half_wall, 0.0),
            Vec3::new(side, thickness, side),
            theme::WHITE,
        ),
        (
            Vec3::new(0.0, reach, -reach - half_wall),
            Vec3::new(side, side, thickness),
            theme::WHITE,
        ),
        (
            Vec3::new(-reach - half_wall, reach, 0.0),
            Vec3::new(thickness, side, side),
            theme::LEFT_WALL,
        ),
        (
            Vec3::new(reach + half_wall, reach, 0.0),
            Vec3::new(thickness, side, side),
            theme::RIGHT_WALL,
        ),
    ];
    for (at, scale, color) in surfaces {
        let surface = spawn_mesh_at(commands, "Cube", at, scale);
        dress(
            commands,
            materials,
            surface,
            Material {
                base_color: color,
                roughness: theme::ROUGHNESS,
                metallic: 0.0,
                ..Default::default()
            },
        );
    }

    let lamp_at = Vec3::new(0.0, side - theme::LAMP_THICKNESS, 0.0);
    let lamp = spawn_mesh_at(
        commands,
        "Cube",
        lamp_at,
        Vec3::new(
            theme::LAMP_REACH * 2.0,
            theme::LAMP_THICKNESS,
            theme::LAMP_REACH * 2.0,
        ),
    );
    dress(
        commands,
        materials,
        lamp,
        Material {
            base_color: [0.0, 0.0, 0.0, 1.0],
            emissive_factor: theme::LAMP_LIGHT,
            unlit: true,
            ..Default::default()
        },
    );
    commands.spawn((
        Name("Lamp".to_string()),
        LocalTransform {
            translation: lamp_at,
            rotation: Quat::identity(),
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
        GlobalTransform::default(),
        Light {
            light_type: LightType::Point,
            color: Vec3::new(1.0, 0.95, 0.85),
            intensity: theme::LAMP_INTENSITY,
            range: theme::LAMP_RANGE,
            cast_shadows: true,
            ..Default::default()
        },
    ));

    let ball = spawn_mesh_at(
        commands,
        "Sphere",
        Vec3::new(0.0, theme::BALL_REACH, 0.0),
        Vec3::new(theme::BALL_REACH, theme::BALL_REACH, theme::BALL_REACH),
    );
    dress(
        commands,
        materials,
        ball,
        Material {
            base_color: theme::WHITE,
            roughness: theme::ROUGHNESS,
            metallic: 0.0,
            ..Default::default()
        },
    );
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
