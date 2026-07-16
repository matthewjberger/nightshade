use crate::ecs::{GameState, TowerDefenseResources, TowerType, register_tower_defense_components};
use crate::retained_ui;
use crate::systems::{
    enemy_movement_system, input_system, load_or_build_level, placement_preview_system,
    projectile_movement_system, range_indicator_system, tile_hover_system, tower_shooting_system,
    tower_targeting_system, ui_update_system, update_money_popups, update_visual_effects,
    wave_spawning_system,
};
use nightshade::prelude::*;

pub struct TowerDefensePlugin;

impl Plugin for TowerDefensePlugin {
    fn build(&self, app: &mut App) {
        app.world.res_mut::<Window>().title = "Tower Defense".to_string();
        app.insert_resource(TowerDefenseResources::default());
        app.add_system(Stage::Startup, initialize);
        app.add_systems(
            Stage::Update,
            (
                retained_ui::poll,
                input_system,
                tile_hover_system,
                placement_preview_system,
                range_indicator_system,
                wave_spawning_system,
                enemy_movement_system,
                tower_targeting_system,
                tower_shooting_system,
                projectile_movement_system,
                visual_effects,
                money_popups,
                ui_update_system,
                camera_reset,
            ),
        );
        app.add_render_graph_config(configure_render_graph);
    }
}

fn initialize(game_world: &mut TowerDefenseResources, world: &mut World) {
    world
        .ecs
        .add_world_at(GAME, register_tower_defense_components());

    world.res_mut::<RenderSettings>().atmosphere = Atmosphere::Nebula;
    world.res_mut::<DebugDraw>().show_grid = false;

    let main_camera = spawn_entities(
        world,
        LOCAL_TRANSFORM | GLOBAL_TRANSFORM | CAMERA | PAN_ORBIT_CAMERA,
        1,
    )[0];

    let camera_radius = 16.1;
    let camera_pitch = 0.52;
    world.set(
        main_camera,
        LocalTransform {
            translation: Vec3::new(0.0, 8.0, 14.0),
            rotation: nalgebra_glm::quat_angle_axis(-0.6, &nalgebra_glm::vec3(1.0, 0.0, 0.0)),
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
    );
    world.set(main_camera, GlobalTransform::default());
    world.set(
        main_camera,
        Camera {
            projection: Projection::Perspective(PerspectiveCamera {
                aspect_ratio: None,
                y_fov_rad: 45.0_f32.to_radians(),
                z_near: 0.1,
                z_far: Some(500.0),
            }),
            smoothing: Some(Smoothing::default()),
        },
    );
    world.set(
        main_camera,
        PanOrbitCamera {
            focus: Vec3::new(0.0, 0.0, 0.0),
            target_focus: Vec3::new(0.0, 0.0, 0.0),
            radius: camera_radius,
            target_radius: camera_radius,
            pitch: camera_pitch,
            target_pitch: camera_pitch,
            yaw: 0.0,
            target_yaw: 0.0,
            ..Default::default()
        },
    );
    world.res_mut::<ActiveCamera>().0 = Some(main_camera);

    game_world.camera_entity = main_camera;
    game_world.money = 200;
    game_world.lives = 20;
    game_world.wave = 0;
    game_world.game_state = GameState::WaitingForWave;
    game_world.selected_tower_type = TowerType::Basic;
    game_world.spawn_timer = 0.0;
    game_world.wave_delay = 3.0;
    game_world.wave_announce_timer = 0.0;
    game_world.game_speed = 1.0;
    game_world.current_hp = 20;
    game_world.max_hp = 20;
    game_world.auto_start_waves = false;

    load_or_build_level(game_world, world);

    retained_ui::build(game_world, world);
}

fn visual_effects(game_world: &mut TowerDefenseResources, world: &mut World) {
    let delta_time = world.res::<Time>().delta_time * game_world.game_speed;
    update_visual_effects(game_world, world, delta_time);
}

fn money_popups(world: &mut World) {
    let popup_delta_time = world.res::<Time>().delta_time;
    update_money_popups(world, popup_delta_time);
}

fn camera_reset(
    mut game_world: ResMut<TowerDefenseResources>,
    reader: EventReader<AppEvent>,
    world: &mut World,
) {
    let game_world = &mut *game_world;
    for event in reader.iter() {
        if let AppEvent::Keyboard {
            key,
            state: KeyState::Pressed,
        } = event
            && (*key == KeyCode::KeyC || *key == KeyCode::Home)
        {
            let camera = game_world.camera_entity;
            if let Some(pan_orbit) =
                world.get_mut::<nightshade::ecs::camera::components::PanOrbitCamera>(camera)
            {
                pan_orbit.target_focus = Vec3::new(0.0, 0.0, 0.0);
                pan_orbit.target_radius = 16.1;
                pan_orbit.target_pitch = 0.52;
                pan_orbit.target_yaw = 0.0;
            }
        }
    }
}

fn configure_render_graph(
    graph: &mut RenderGraph<nightshade::prelude::RenderInputs>,
    device: &wgpu::Device,
    surface_format: wgpu::TextureFormat,
    resources: RenderResources,
) {
    let particle_pass = passes::ParticlePass::new(device, wgpu::TextureFormat::Rgba16Float);
    render_graph_pass(graph, Box::new(particle_pass))
        .slot("color", resources.scene_color)
        .slot("depth", resources.depth)
        .add()
        .unwrap();

    let (width, height) = (1920, 1080);
    let bloom_width = width / 2;
    let bloom_height = height / 2;

    let bloom_texture = render_graph_add_color_texture(graph, "bloom")
        .format(wgpu::TextureFormat::Rgba16Float)
        .size(bloom_width, bloom_height)
        .clear_color(wgpu::Color::BLACK)
        .transient();

    let bloom_pass = passes::BloomPass::new(device, width, height);
    render_graph_pass(graph, Box::new(bloom_pass))
        .read("hdr", resources.scene_color)
        .write("bloom", bloom_texture)
        .add()
        .unwrap();

    let postprocess_pass = passes::PostProcessPass::new(device, surface_format, 0.005);
    render_graph_pass(graph, Box::new(postprocess_pass))
        .read("hdr", resources.scene_color)
        .read("scene_color", resources.scene_color)
        .read("bloom", bloom_texture)
        .read("ssao", resources.ssao)
        .write("output", resources.compute_output)
        .add()
        .unwrap();

    let aa_output = render_graph_add_color_texture(graph, "aa_output")
        .format(surface_format)
        .size(
            resources.surface_width.max(1),
            resources.surface_height.max(1),
        )
        .transient();

    let taa_pass = passes::TaaPass::new(device, surface_format);
    render_graph_pass(graph, Box::new(taa_pass))
        .read("input", resources.compute_output)
        .read("depth", resources.depth)
        .write("output", aa_output)
        .add()
        .unwrap();

    let swapchain_blit_pass =
        passes::BlitPass::new(device, surface_format).with_name("default_swapchain_blit");
    render_graph_pass(graph, Box::new(swapchain_blit_pass))
        .read("input", aa_output)
        .write("output", resources.swapchain)
        .add()
        .unwrap();
}
