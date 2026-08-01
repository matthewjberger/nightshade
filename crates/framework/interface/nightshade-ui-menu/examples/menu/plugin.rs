use crate::systems;
use crate::theme;
use nightshade::app::{App, Stage};
use nightshade::plugin::Plugin;

pub struct ExamplePlugin;

impl Plugin for ExamplePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(crate::resources::Buttons::default());
        app.add_plugin(nightshade_state::StatePlugin::new(
            crate::resources::Screen::Title,
        ));
        app.add_plugin(nightshade_camera_controller::CameraControllerPlugin);
        app.add_plugin(nightshade_readout::ReadoutPlugin::titled(theme::TITLE));
        app.add_plugin(nightshade_ui::UiPlugin::new());
        app.add_plugin(
            nightshade_ui_menu::MenuPlugin::<crate::resources::Screen>::new(
                nightshade_ui_menu::data::dark_menu_style(),
            ),
        );
        app.add_system(Stage::Startup, systems::setup::stand_it_up);
        app.add_system(Stage::Startup, build_the_menus);
        app.add_system(Stage::First, systems::play::take_the_clicks);
        app.add_system(Stage::First, systems::play::pause_and_play);
        app.add_system(Stage::Update, systems::play::show_the_screen);
    }
}

fn build_the_menus(
    style: nightshade::ecs::system_param::Res<nightshade_ui_menu::data::MenuStyle>,
    mut menus: nightshade::ecs::system_param::ResMut<
        nightshade_ui_menu::resources::Menus<crate::resources::Screen>,
    >,
    mut buttons: nightshade::ecs::system_param::ResMut<crate::resources::Buttons>,
    mut build: nightshade_ui::commands::builder::UiBuild,
) {
    let mut tree = nightshade_ui::prelude::UiTreeBuilder::new(build.context());
    let title = nightshade_ui_menu::commands::add_menu(
        &mut tree,
        &style,
        theme::PANEL_WIDTH,
        theme::PANEL_HEIGHT,
        "The Menu",
        &["A panel with a heading, some rows and a few buttons."],
        &["Play", "Quit"],
    );
    let paused = nightshade_ui_menu::commands::add_menu(
        &mut tree,
        &style,
        theme::PANEL_WIDTH,
        theme::PANEL_HEIGHT,
        "Paused",
        &[],
        &["Resume", "Quit to title"],
    );
    let over = nightshade_ui_menu::commands::add_menu(
        &mut tree,
        &style,
        theme::PANEL_WIDTH,
        theme::PANEL_HEIGHT,
        "Over",
        &["The crate shows one panel for each screen."],
        &["Again"],
    );
    tree.finish();

    buttons.play = title.buttons[0];
    buttons.quit = title.buttons[1];
    buttons.resume = paused.buttons[0];
    buttons.to_title = paused.buttons[1];
    buttons.again = over.buttons[0];

    nightshade_ui_menu::commands::menu_show_on(
        &mut menus,
        crate::resources::Screen::Title,
        title.panel,
    );
    nightshade_ui_menu::commands::menu_show_on(
        &mut menus,
        crate::resources::Screen::Paused,
        paused.panel,
    );
    nightshade_ui_menu::commands::menu_show_on(
        &mut menus,
        crate::resources::Screen::Over,
        over.panel,
    );
}
