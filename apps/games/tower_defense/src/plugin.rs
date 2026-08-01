use crate::data::{Ask, GameState, TowerType};
use crate::resources::TowerDefenseResources;
use crate::systems::{
    build_level, enemy_movement_system, input_system, link_level, placement_preview_system,
    projectile_movement_system, range_indicator_system, tile_hover_system, tower_shooting_system,
    tower_targeting_system, ui_update_system, update_money_popups, update_visual_effects,
    wave_spawning_system,
};
use nightshade::prelude::*;
use nightshade_camera::prelude::*;
use nightshade_camera_controller::contract::*;
use nightshade_graphics::prelude::*;
use nightshade_input::{Actions, InputPlugin};
use nightshade_platform::prelude::*;

pub struct TowerDefensePlugin;

impl Plugin for TowerDefensePlugin {
    fn build(&self, app: &mut App) {
        app.world.res_mut::<Window>().title = "Tower Defense".to_string();
        app.insert_resource(TowerDefenseResources::default());
        app.add_plugin(InputPlugin::<Ask>::new());
        app.add_system(Stage::Startup, initialize);
        app.add_system(Stage::Startup, link_level_tables);
        app.add_system(Stage::Startup, build_ui);
        app.add_system(
            Stage::Update,
            (
                poll_ui,
                input,
                tile_hover,
                placement_preview,
                range_indicator,
                wave_spawning,
                enemy_movement,
                tower_targeting,
                tower_shooting,
                projectile_movement,
                visual_effects,
                money_popups,
                update_ui,
                camera_reset,
            ),
        );
    }
}

fn initialize(
    mut game_world: ResMut<TowerDefenseResources>,
    mut settings: ResMut<RenderSettings>,
    mut debug_draw: ResMut<DebugDraw>,
    mut active_camera: ResMut<ActiveCamera>,
    mut parts: crate::commands::SceneParts,
    storage: &mut nightshade::ecs::Storage,
) {
    let game_world = &mut *game_world;
    let mut scene = parts.scene(storage);
    settings.atmosphere = Atmosphere::Nebula;
    debug_draw.show_grid = false;

    let main_camera = scene.commands.spawn(());

    let camera_radius = 16.1;
    let camera_pitch = 0.52;
    scene.commands.set(
        main_camera,
        LocalTransform {
            translation: Vec3::new(0.0, 8.0, 14.0),
            rotation: nalgebra_glm::quat_angle_axis(-0.6, &nalgebra_glm::vec3(1.0, 0.0, 0.0)),
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
    );
    scene.commands.set(main_camera, GlobalTransform::default());
    scene.commands.set(
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
    scene.commands.set(
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
    active_camera.0 = Some(main_camera);

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
    build_level(&mut scene);
}

fn link_level_tables(
    mut game_world: ResMut<TowerDefenseResources>,
    mut parts: crate::commands::SceneParts,
    storage: &mut nightshade::ecs::Storage,
) {
    link_level(&mut game_world, &mut parts.scene(storage));
}

fn build_ui(
    mut game_world: ResMut<TowerDefenseResources>,
    mut build: nightshade_ui::commands::builder::UiBuild,
) {
    crate::commands::build(&mut game_world, &mut build);
}

fn step(time: &Time, game_world: &TowerDefenseResources) -> f32 {
    time.delta_time * game_world.game_speed
}

fn input(
    mut game_world: ResMut<TowerDefenseResources>,
    input: Res<Input>,
    actions: Res<Actions<Ask>>,
    window: Res<Window>,
    viewport: Res<nightshade_render::resources::viewport::Viewport>,
    active_camera: Res<ActiveCamera>,
    runtime: Res<nightshade_ui::resources::RetainedUiRuntime>,
    interaction: Res<nightshade_ui::resources::RetainedUiInteraction>,
    mut parts: crate::commands::SceneParts,
    storage: &mut nightshade::ecs::Storage,
) {
    let view = crate::systems::MouseView {
        window: &window,
        viewport: &viewport,
        active_camera: &active_camera,
    };
    let pointer_over_ui =
        nightshade_ui::ui_pointer_target_of(&runtime, &interaction, storage).is_some();
    let game_world = &mut *game_world;
    input_system(
        game_world,
        &input,
        &actions,
        &view,
        pointer_over_ui,
        &mut parts.scene(storage),
    );
}

fn tile_hover(
    mut game_world: ResMut<TowerDefenseResources>,
    mut parts: crate::commands::SceneParts,
    storage: &mut nightshade::ecs::Storage,
) {
    tile_hover_system(&mut game_world, &mut parts.scene(storage));
}

fn placement_preview(
    mut game_world: ResMut<TowerDefenseResources>,
    mut parts: crate::commands::SceneParts,
    storage: &mut nightshade::ecs::Storage,
) {
    placement_preview_system(&mut game_world, &mut parts.scene(storage));
}

fn range_indicator(
    mut game_world: ResMut<TowerDefenseResources>,
    mut parts: crate::commands::SceneParts,
    storage: &mut nightshade::ecs::Storage,
) {
    range_indicator_system(&mut game_world, &mut parts.scene(storage));
}

fn wave_spawning(
    mut game_world: ResMut<TowerDefenseResources>,
    time: Res<Time>,
    mut parts: crate::commands::SceneParts,
    storage: &mut nightshade::ecs::Storage,
) {
    let delta_time = step(&time, &game_world);
    wave_spawning_system(&mut game_world, &mut parts.scene(storage), delta_time);
}

fn enemy_movement(
    mut game_world: ResMut<TowerDefenseResources>,
    time: Res<Time>,
    mut parts: crate::commands::SceneParts,
    storage: &mut nightshade::ecs::Storage,
) {
    let delta_time = step(&time, &game_world);
    enemy_movement_system(&mut game_world, &mut parts.scene(storage), delta_time);
}

fn tower_targeting(
    mut game_world: ResMut<TowerDefenseResources>,
    time: Res<Time>,
    mut parts: crate::commands::SceneParts,
    storage: &mut nightshade::ecs::Storage,
) {
    let delta_time = step(&time, &game_world);
    tower_targeting_system(&mut game_world, &mut parts.scene(storage), delta_time);
}

fn tower_shooting(
    mut game_world: ResMut<TowerDefenseResources>,
    time: Res<Time>,
    mut parts: crate::commands::SceneParts,
    storage: &mut nightshade::ecs::Storage,
) {
    let delta_time = step(&time, &game_world);
    tower_shooting_system(&mut game_world, &mut parts.scene(storage), delta_time);
}

fn projectile_movement(
    mut game_world: ResMut<TowerDefenseResources>,
    time: Res<Time>,
    mut parts: crate::commands::SceneParts,
    storage: &mut nightshade::ecs::Storage,
) {
    let delta_time = step(&time, &game_world);
    projectile_movement_system(&mut game_world, &mut parts.scene(storage), delta_time);
}

fn visual_effects(
    mut game_world: ResMut<TowerDefenseResources>,
    time: Res<Time>,
    mut parts: crate::commands::SceneParts,
    storage: &mut nightshade::ecs::Storage,
) {
    let delta_time = step(&time, &game_world);
    update_visual_effects(&mut game_world, &mut parts.scene(storage), delta_time);
}

fn money_popups(
    time: Res<Time>,
    mut parts: crate::commands::SceneParts,
    storage: &mut nightshade::ecs::Storage,
) {
    let popup_delta_time = time.delta_time;
    update_money_popups(&mut parts.scene(storage), popup_delta_time);
}

fn poll_ui(
    mut game_world: ResMut<TowerDefenseResources>,
    frame: Res<nightshade_ui::resources::RetainedUiFrame>,
) {
    crate::commands::poll(&mut game_world, &frame);
}

fn update_ui(
    mut game_world: ResMut<TowerDefenseResources>,
    time: Res<Time>,
    mut build: nightshade_ui::commands::builder::UiBuild,
) {
    let delta_time = step(&time, &game_world);
    ui_update_system(&mut game_world, &mut build, delta_time);
}

fn camera_reset(
    game_world: Res<TowerDefenseResources>,
    actions: Res<Actions<Ask>>,
    mut cameras: Query<(&mut nightshade_camera_controller::contract::PanOrbitCamera,)>,
) {
    if !actions.just_pressed(Ask::ResetCamera) {
        return;
    }
    cameras.for_entity(game_world.camera_entity, |(pan_orbit,)| {
        pan_orbit.target_focus = Vec3::new(0.0, 0.0, 0.0);
        pan_orbit.target_radius = 16.1;
        pan_orbit.target_pitch = 0.52;
        pan_orbit.target_yaw = 0.0;
    });
}
