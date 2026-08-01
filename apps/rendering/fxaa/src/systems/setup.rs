use crate::resources::{Opening, Panel};
use crate::theme;
use nightshade::prelude::*;
use nightshade_camera::prelude::ActiveCamera;
use nightshade_camera_controller::commands::spawn_pan_orbit_camera_at;
use nightshade_graphics::prelude::*;

pub fn initialize(
    mut render_settings: ResMut<RenderSettings>,
    opening: Res<Opening>,
    mut active_camera: ResMut<ActiveCamera>,
    mut panel_state: ResMut<Panel>,
    mut materials: ResMut<MaterialRegistry>,
    mut build: nightshade_ui::commands::builder::UiBuild,
) {
    settle_the_look(&mut render_settings);
    let smoothing_on = opening.smoothing_on;
    let camera = {
        let commands = build.context().commands;
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
    let panel = crate::systems::panel::build(build.context(), smoothing_on);
    active_camera.0 = Some(camera);
    *panel_state = panel;
}

fn settle_the_look(settings: &mut RenderSettings) {
    settings.show_sky = false;
    settings.clear_color = [0.012, 0.013, 0.016, 1.0];
    settings.bloom_enabled = false;
    settings.render_scale = 1.0 / theme::PIXELS_PER_STEP;
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
            metallic: 0.0,
            ..Default::default()
        },
    );

    for index in 0..theme::BARS {
        let turn = std::f32::consts::PI * index as f32 / theme::BARS as f32;
        let bar = spawn_mesh_at(
            commands,
            "Cube",
            Vec3::new(0.0, theme::BAR_HEIGHT, 0.0),
            Vec3::new(
                theme::BAR_LENGTH,
                theme::BAR_THICKNESS,
                theme::BAR_THICKNESS,
            ),
        );
        turn_around_the_eye(commands, bar, turn);
        dress(
            commands,
            materials,
            bar,
            Material {
                base_color: [0.0, 0.0, 0.0, 1.0],
                emissive_factor: theme::BAR_COLOR,
                unlit: true,
                ..Default::default()
            },
        );
    }

    for (across, turn) in theme::BLOCKS {
        let block = spawn_mesh_at(
            commands,
            "Cube",
            Vec3::new(across, theme::BLOCK_REACH, theme::BLOCK_BACK),
            Vec3::new(theme::BLOCK_REACH, theme::BLOCK_REACH, theme::BLOCK_REACH),
        );
        turn_upright(commands, block, turn);
        dress(
            commands,
            materials,
            block,
            Material {
                base_color: theme::BLOCK_COLOR,
                roughness: theme::BLOCK_ROUGHNESS,
                metallic: 0.0,
                ..Default::default()
            },
        );
    }
}

fn turn_around_the_eye(
    commands: &mut nightshade::ecs::system_param::Commands<'_>,
    entity: nightshade::ecs::Entity,
    turn: f32,
) {
    set_rotation(
        commands,
        entity,
        nalgebra_glm::quat_angle_axis(turn, &Vec3::z()),
    );
}

fn turn_upright(
    commands: &mut nightshade::ecs::system_param::Commands<'_>,
    entity: nightshade::ecs::Entity,
    turn: f32,
) {
    set_rotation(
        commands,
        entity,
        nalgebra_glm::quat_angle_axis(turn, &Vec3::y()),
    );
}

fn set_rotation(
    commands: &mut nightshade::ecs::system_param::Commands<'_>,
    entity: nightshade::ecs::Entity,
    rotation: Quat,
) {
    commands.queue(move |storage, _| {
        if let Some(transform) = storage.get_mut::<LocalTransform>(entity) {
            transform.rotation = rotation;
        }
    });
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
