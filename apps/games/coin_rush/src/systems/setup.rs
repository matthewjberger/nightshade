use crate::resources::{CoinRushResources, Screen, UiHandles};
use crate::systems::round::random_spot;
use crate::systems::screens::set_screen;
use crate::systems::textures::{self, FLOOR_TEXTURE};
use crate::theme::*;
use nightshade::prelude::*;
use nightshade_camera::commands::scene::spawn_camera;
use nightshade_camera::prelude::*;
use nightshade_camera::queries::camera_look::camera_look_rotation;
use nightshade_graphics::commands::builtins::spawn_mesh_at;
use nightshade_graphics::prelude::*;
use nightshade_hover::{Bob, Perch, Phase, Spin};
use nightshade_pen::Pen;
use nightshade_pickup::{Collector, Pickup, PickupCount, Taken};
use nightshade_render::commands::material_registry::insert_material;
use nightshade_render::components::material::MaterialRef;
use nightshade_ui::prelude::*;
use nightshade_ui_banner::prelude::*;
use nightshade_ui_menu::prelude::*;

const COIN_COUNT: usize = 8;
const FLOOR_TILING: f32 = 8.0;
const CAMERA_EYE: Vec3 = Vec3::new(0.0, 24.0, 16.0);

pub fn initialize(
    mut resources: ResMut<CoinRushResources>,
    mut settings: ResMut<RenderSettings>,
    mut registry: ResMut<MaterialRegistry>,
    mut dirty: ResMut<nightshade_ui::resources::RetainedUiDirty>,
    mut commands: Commands,
    mut textures_load: nightshade_graphics::resources::loading::TextureLoad,
) {
    settings.atmosphere = Atmosphere::Sky;
    spawn_sun(&mut commands);

    textures::load_prototype_textures(&mut textures_load);
    spawn_floor(&mut registry, &mut commands);
    spawn_look_camera(&mut commands);
    spawn_player(&mut registry, &mut commands);
    spawn_coins(&mut resources, &mut registry, &mut commands);

    set_screen(&mut resources, &mut dirty, &mut commands, Screen::Title);
}

fn spawn_floor(registry: &mut MaterialRegistry, commands: &mut Commands) {
    let half_extent = ARENA + 2.0;
    let entity = spawn_mesh_at(
        commands,
        "Plane",
        Vec3::zeros(),
        Vec3::new(half_extent, 1.0, half_extent),
    );
    let material = insert_material(
        registry,
        "coin_rush_floor",
        textures::tiled_material(FLOOR_TEXTURE, FLOOR_TILING),
    );
    commands.set(entity, MaterialRef::new(material));
}

fn spawn_look_camera(commands: &mut Commands) {
    let camera = spawn_camera(commands, CAMERA_EYE, "Main Camera".to_string());
    if let Some(transform) = commands.staged_mut::<LocalTransform>(camera) {
        transform.rotation = camera_look_rotation(CAMERA_EYE, Vec3::zeros(), 0.0);
    }
    commands.restage::<LocalTransform>(camera);
    commands.queue(move |_, resources| {
        resources.res_mut::<ActiveCamera>().0 = Some(camera);
    });
}

fn spawn_player(registry: &mut MaterialRegistry, commands: &mut Commands) {
    let entity = spawn_mesh_at(
        commands,
        "Sphere",
        Vec3::new(0.0, 0.5, 0.0),
        Vec3::new(1.0, 1.0, 1.0),
    );
    let material = insert_material(
        registry,
        "coin_rush_player",
        textures::solid_material(PLAYER_COLOR, 0.4, 0.1),
    );
    commands.set(entity, MaterialRef::new(material));
    commands.set(entity, nightshade_topdown::Walking(1.0));
    commands.set(entity, Collector(COIN_GRASP));
    commands.set(entity, Pen(nalgebra_glm::vec2(ARENA, ARENA)));
    commands.set(
        entity,
        nightshade_topdown::Heading(nalgebra_glm::vec2(0.0, 0.0)),
    );
}

fn spawn_coins(
    resources: &mut CoinRushResources,
    registry: &mut MaterialRegistry,
    commands: &mut Commands,
) {
    let material = insert_material(
        registry,
        "coin_rush_coin",
        textures::solid_material(COIN_COLOR, 0.3, 0.6),
    );
    for index in 0..COIN_COUNT {
        let spot = random_spot(&mut resources.round.roll);
        let entity = spawn_mesh_at(commands, "Cylinder", spot, Vec3::new(0.7, 0.12, 0.7));
        commands.set(entity, MaterialRef::new(material));
        commands.set(entity, Perch(spot));
        commands.set(entity, Bob(COIN_BOB));
        commands.set(entity, Phase(index as f32 * COIN_PHASE));
        commands.set(entity, Spin(COIN_SPIN));
        commands.set(entity, Pickup(COIN_KIND));
        commands.set(entity, PickupCount(1));
        commands.set(entity, Taken(false));
        if let Some(transform) = commands.staged_mut::<LocalTransform>(entity) {
            transform.rotation =
                nalgebra_glm::quat_angle_axis(std::f32::consts::FRAC_PI_2, &Vec3::x());
        }
        commands.restage::<LocalTransform>(entity);
    }
}

pub fn build_overlays(
    mut resources: ResMut<CoinRushResources>,
    mut handles: ResMut<BannerHandles>,
    mut build: nightshade_ui::commands::builder::UiBuild,
) {
    let style = menu_style();
    let title_style = title_style();
    let mut ui = UiHandles::default();
    let mut banner = BannerHandles::default();
    {
        let mut tree = UiTreeBuilder::new(build.context());
        let root = tree
            .add_node()
            .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
            .entity();

        tree.in_parent(root, |tree| {
            banner = add_banner(tree, &banner_style());

            let title = add_menu(
                tree,
                &title_style,
                TITLE_WIDTH,
                TITLE_HEIGHT,
                "Coin Rush",
                &[
                    "Reach the coin goal before time runs out.",
                    "WASD, arrows, or a gamepad. P to pause.",
                ],
                &["Play", "Quit"],
            );
            ui.title = Some(title.panel);
            ui.play_button = Some(title.buttons[0]);
            ui.quit_button = Some(title.buttons[1]);

            let pause = add_menu(
                tree,
                &style,
                MENU_WIDTH,
                MENU_HEIGHT,
                "Paused",
                &[],
                &["Resume", "Restart", "Quit to title"],
            );
            ui.pause = Some(pause.panel);
            ui.resume_button = Some(pause.buttons[0]);
            ui.restart_button = Some(pause.buttons[1]);
            ui.pause_title_button = Some(pause.buttons[2]);

            let over = add_menu(
                tree,
                &style,
                MENU_WIDTH,
                MENU_HEIGHT,
                "Time!",
                &["Score 0"],
                &["Play again", "Quit to title"],
            );
            ui.over = Some(over.panel);
            ui.over_score = Some(over.rows[0]);
            ui.again_button = Some(over.buttons[0]);
            ui.over_title_button = Some(over.buttons[1]);
        });

        tree.finish();
    }
    resources.ui = ui;
    *handles = banner;
}

fn menu_style() -> MenuStyle {
    MenuStyle {
        background: PANEL_BG,
        border: PANEL_BORDER,
        corner_radius: 8.0,
        border_width: 1.0,
        padding: 12.0,
        spacing: 8.0,
        visible: true,
        centered: true,
        heading_size: HEADING_SIZE,
        heading_color: TEXT_PRIMARY,
        row_size: LABEL_SIZE,
        row_color: TEXT_PRIMARY,
        ..Default::default()
    }
}

fn title_style() -> MenuStyle {
    MenuStyle {
        heading_size: TITLE_SIZE,
        ..menu_style()
    }
}

fn banner_style() -> BannerStyle {
    BannerStyle {
        size: LABEL_SIZE,
        color: TEXT_PRIMARY,
        ..Default::default()
    }
}
