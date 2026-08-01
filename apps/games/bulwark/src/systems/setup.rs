use crate::commands::{art, board};
use crate::data::{self, Settings};
use crate::queries;
use crate::resources::Bulwark;
use crate::systems::{hud, screens};
use crate::theme;
use nightshade::prelude::*;
use nightshade_camera::prelude::*;
use nightshade_config::contract::Config;
use nightshade_graphics::prelude::*;
use nightshade_sprite::prelude::SpriteAtlases;
use nightshade_ui_config::prelude::ConfigPanel;
use nightshade_ui_menu::prelude::{MenuStyle, Menus};
use nightshade_waves::{WavePlan, WaveProgress};

pub fn initialize(
    mut settings: ResMut<RenderSettings>,
    mut atlases: ResMut<SpriteAtlases>,
    mut active_camera: ResMut<ActiveCamera>,
    mut bulwark: ResMut<Bulwark>,
    arguments: Res<crate::resources::Arguments>,
    config: Res<Config<Settings>>,
    style: Res<MenuStyle>,
    mut menus: ResMut<Menus<crate::resources::Screen>>,
    mut rows: ResMut<ConfigPanel<Settings>>,
    mut build: nightshade_ui::commands::builder::UiBuild,
    mut commands: Commands,
) {
    settings.atmosphere = Atmosphere::None;
    settings.clear_color = theme::VOID_COLOR;
    spawn_sun_without_shadows(&mut commands);

    let packed = art::pack_the_art(&mut atlases);
    bulwark.board = arguments.board.min(queries::boards_total() - 1);

    board::raise_the_backdrop(&mut commands, &packed);
    bulwark.marker = Some(board::raise_the_marker(&mut commands, &packed));
    bulwark.ring = Some(board::raise_the_ring(&mut commands, &packed));
    bulwark.ghost = Some(board::raise_the_ghost(&mut commands, &packed));
    bulwark.taken = vec![None; (data::COLUMNS * data::ROWS) as usize];
    bulwark.art = packed;
    let opening = queries::opening_wave(arguments.wave);
    bulwark.coin =
        queries::opening_coin(config.choice(Settings::Difficulty)) + queries::skipped_coin(opening);
    bulwark.lives = theme::STARTING_LIVES;
    bulwark.best = queries::best_run();
    if arguments.wall > 0 {
        board::raise_a_wall(&mut commands, &mut bulwark, opening, arguments.wall);
    }

    let camera = nightshade_camera::commands::scene::spawn_flat_camera(
        &mut commands,
        "Board Camera".to_string(),
        theme::CAMERA_AT,
        theme::CAMERA_HALF_HEIGHT,
        theme::CAMERA_NEAR,
        theme::CAMERA_FAR,
    );
    active_camera.0 = Some(camera);
    bulwark.camera = Some(camera);

    let mut tree = nightshade_ui::prelude::UiTreeBuilder::new(build.context());
    bulwark.hud = hud::build(&mut tree);
    bulwark.screens = screens::build(&mut tree, &style, &mut menus, &config, &mut rows);
    tree.finish();
}

pub fn menu_style() -> MenuStyle {
    MenuStyle {
        background: nightshade::prelude::Vec4::from(theme::HUD_PANEL),
        border: nightshade::prelude::Vec4::from(theme::HUD_BORDER),
        heading_color: nightshade::prelude::Vec4::from(theme::HUD_VALUE),
        row_color: nightshade::prelude::Vec4::from(theme::HUD_DIM),
        centered: true,
        padding: 18.0,
        spacing: 12.0,
        ..Default::default()
    }
}

pub fn lay_the_plan(
    arguments: Res<crate::resources::Arguments>,
    config: Res<Config<Settings>>,
    mut plan: ResMut<WavePlan>,
    mut progress: ResMut<WaveProgress>,
) {
    let rest = config.number(Settings::Rest);
    plan.0 = queries::wave_plan(rest);
    progress.wave = queries::opening_wave(arguments.wave);
    progress.resting = true;
    progress.rest = rest * theme::FIRST_REST_SHARE;
}
