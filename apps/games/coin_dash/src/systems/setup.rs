use crate::components::{Home, Phase};
use crate::data::{Ground, MenuCamera};
use crate::resources::{CoinDashResources, Screen, UiHandles};
use crate::systems::materials;
use crate::systems::screens::{refresh, set_cursor_locked};
use crate::systems::textures::{self, GROUND_GREEN};
use crate::systems::world::{ARENA, COIN_COUNT, COIN_HEIGHT, generate_world};
use crate::theme::*;
use nightshade::prelude::*;
use nightshade_camera::commands::scene::spawn_camera;
use nightshade_camera::prelude::*;
use nightshade_camera::queries::camera_look::camera_look_rotation;
use nightshade_graphics::commands::builtins::spawn_mesh_at;
use nightshade_graphics::prelude::*;
use nightshade_physics::contract::components::{ColliderComponent, RigidBodyComponent};
use nightshade_platform::prelude::*;
use nightshade_render::data::config::Fog;
use nightshade_ui::prelude::*;
use nightshade_ui_menu::prelude::*;

const FLOOR_HALF_EXTENT: f32 = ARENA + 10.0;
const CAMERA_EYE: Vec3 = Vec3::new(0.0, 20.0, 34.0);
const CAMERA_TARGET: Vec3 = Vec3::new(0.0, 2.0, 0.0);

pub fn initialize(
    mut resources: ResMut<CoinDashResources>,
    mut settings: ResMut<RenderSettings>,
    mut window: ResMut<Window>,
    mut dirty: ResMut<nightshade_ui::resources::RetainedUiDirty>,
    mut commands: Commands,
    mut textures_load: nightshade_graphics::resources::loading::TextureLoad,
) {
    settings.atmosphere = Atmosphere::Sky;
    settings.fog = Some(Fog {
        color: [0.7, 0.8, 0.92],
        start: 28.0,
        end: ARENA * 1.9,
        ..Fog::default()
    });

    let sun = spawn_sun(&mut commands);
    commands.queue(move |storage, _| {
        if let Some(light) = storage.get_mut::<Light>(sun) {
            light.color = Vec3::new(1.0, 0.96, 0.9);
            light.intensity = 3.2;
        }
    });

    textures::load_prototype_textures(&mut textures_load);
    spawn_floor(&mut commands);
    spawn_coins(&mut commands);
    spawn_menu_camera(&mut commands);

    generate_world(&mut resources, &mut commands);
    set_cursor_locked(&mut window, false);
    resources.round.screen = Screen::Title;
    refresh(&resources, &mut dirty, &mut commands);
}

fn spawn_floor(commands: &mut Commands) {
    let entity = spawn_mesh_at(
        commands,
        "Plane",
        Vec3::zeros(),
        Vec3::new(FLOOR_HALF_EXTENT, 1.0, FLOOR_HALF_EXTENT),
    );
    materials::attach(
        commands,
        entity,
        "coin_dash_ground_green",
        textures::tiled_material(GROUND_GREEN, 14.0),
    );
    commands.add_tag_type::<Ground>(entity);
    commands.set(
        entity,
        RigidBodyComponent::new_static().with_translation(0.0, -0.05, 0.0),
    );
    commands.set(
        entity,
        ColliderComponent::new_cuboid(FLOOR_HALF_EXTENT, 0.05, FLOOR_HALF_EXTENT)
            .with_friction(0.8)
            .with_restitution(0.1),
    );
}

fn spawn_coins(commands: &mut Commands) {
    let material = "coin_dash_coin";
    for index in 0..COIN_COUNT {
        let entity = spawn_mesh_at(
            commands,
            "Cylinder",
            Vec3::new(0.0, COIN_HEIGHT, 0.0),
            Vec3::new(0.7, 0.12, 0.7),
        );
        materials::attach(
            commands,
            entity,
            material,
            textures::solid_material(GOLD, 0.3, 0.6),
        );
        commands.set(entity, Home(Vec3::new(0.0, COIN_HEIGHT, 0.0)));
        commands.set(entity, Phase(index as f32 * 0.7));
        if let Some(transform) = commands.staged_mut::<LocalTransform>(entity) {
            transform.rotation =
                nalgebra_glm::quat_angle_axis(std::f32::consts::FRAC_PI_2, &Vec3::x());
        }
        commands.restage::<LocalTransform>(entity);
    }
}

fn spawn_menu_camera(commands: &mut Commands) {
    let camera = spawn_camera(commands, CAMERA_EYE, "Menu Camera".to_string());
    commands.queue(move |storage, resources| {
        if let Some(transform) = storage.get_mut::<LocalTransform>(camera) {
            transform.rotation = camera_look_rotation(CAMERA_EYE, CAMERA_TARGET, 0.0);
        }
        resources.res_mut::<ActiveCamera>().0 = Some(camera);
    });
    commands.add_tag_type::<MenuCamera>(camera);
}

pub fn build_overlays(
    mut resources: ResMut<CoinDashResources>,
    mut build: nightshade_ui::commands::builder::UiBuild,
) {
    let handles = {
        let mut tree = UiTreeBuilder::new(build.context());
        let root = tree
            .add_node()
            .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
            .entity();

        let handles = tree.in_parent(root, |tree| {
            let mut handles = UiHandles::default();

            let hud = tree
                .add_node()
                .window(
                    Rl(vec2(50.0, 0.0)) + Ab(vec2(0.0, 20.0)),
                    Ab(vec2(HUD_WIDTH, HUD_HEIGHT)),
                    Anchor::TopCenter,
                )
                .with_rect(8.0, 1.0, PANEL_BORDER)
                .color_raw::<UiBase>(PANEL_BG)
                .flow(FlowDirection::Vertical, 12.0, 8.0)
                .entity();
            handles.hud = Some(hud);
            tree.in_parent(hud, |tree| {
                handles.hud_label = Some(text_row(tree, "", HUD_SIZE));
            });

            let title = add_menu_panel(tree, &menu_style(), TITLE_WIDTH, TITLE_HEIGHT);
            handles.title = Some(title);
            tree.in_parent(title, |tree| {
                text_row(tree, "Coin Dash", TITLE_SIZE);
                text_row(
                    tree,
                    "Run through procedural worlds and grab every coin.",
                    LABEL_SIZE,
                );
                text_row(
                    tree,
                    "WASD, mouse, or a gamepad. Shift sprints, Space jumps, P pauses.",
                    LABEL_SIZE,
                );
                handles.play_button = Some(tree.add_button("Play"));
                handles.quit_button = Some(tree.add_button("Quit"));
            });

            let pause = add_menu_panel(tree, &menu_style(), PAUSE_WIDTH, PAUSE_HEIGHT);
            handles.pause = Some(pause);
            tree.in_parent(pause, |tree| {
                text_row(tree, "Paused", HEADING_SIZE);
                handles.resume_button = Some(tree.add_button("Resume"));
                handles.restart_button = Some(tree.add_button("Restart"));
                handles.pause_title_button = Some(tree.add_button("Quit to title"));
            });

            let over = add_menu_panel(tree, &menu_style(), OVER_WIDTH, OVER_HEIGHT);
            handles.over = Some(over);
            tree.in_parent(over, |tree| {
                text_row(tree, "Time!", HEADING_SIZE);
                handles.over_score = Some(text_row(tree, "", LABEL_SIZE));
                handles.again_button = Some(tree.add_button("Play again"));
                handles.over_title_button = Some(tree.add_button("Quit to title"));
            });

            handles
        });

        tree.finish();
        handles
    };
    resources.ui = handles;
}

fn text_row(tree: &mut UiTreeBuilder, text: &str, size: f32) -> Entity {
    tree.add_node()
        .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, size + ROW_HEIGHT - LABEL_SIZE)))
        .with_text(text, size)
        .color_raw::<UiBase>(TEXT_PRIMARY)
        .entity()
}

pub fn menu_style() -> MenuStyle {
    MenuStyle {
        background: PANEL_BG,
        border: PANEL_BORDER,
        corner_radius: 8.0,
        border_width: 1.0,
        padding: 12.0,
        spacing: 8.0,
        visible: true,
        ..Default::default()
    }
}
