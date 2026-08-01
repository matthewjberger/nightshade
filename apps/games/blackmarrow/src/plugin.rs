use crate::data::ask::Ask;
use crate::data::settings::Settings;
use crate::data::theme;
use crate::resources::{BlackmarrowResources, Console, Screen};
use crate::systems;
use nightshade::prelude::*;
use nightshade_config::prelude::ConfigPlugin;
use nightshade_input::InputPlugin;
use nightshade_platform::prelude::*;
use nightshade_shell::ShellOverlayPlugin;
use nightshade_state::prelude::*;
use nightshade_ui_menu::prelude::MenuPlugin;

pub struct BlackmarrowPlugin;

impl Plugin for BlackmarrowPlugin {
    fn build(&self, app: &mut App) {
        app.world.res_mut::<Window>().title = theme::TITLE.to_string();
        app.insert_resource(BlackmarrowResources::default());
        app.add_plugin(StatePlugin::new(Screen::Title));
        app.add_plugin(InputPlugin::<Ask>::new());
        app.add_plugin(MenuPlugin::<Screen>::new(systems::hud::menu_style()));
        app.add_plugin(ConfigPlugin::<Settings>::new().persist("nightshade", "blackmarrow.toml"));
        app.insert_resource(systems::shell::new_shell());
        app.add_plugin(ShellOverlayPlugin::<Console>::default());

        app.add_system(Stage::Startup, systems::setup::initialize);
        app.add_system(Stage::Startup, systems::station::build);
        app.add_system(Stage::Startup, systems::props::request);

        app.add_system(Stage::Update, systems::shell::read_the_keys);
        app.add_system(Stage::Update, systems::shell::route_the_asks);
        app.add_system(Stage::Update, systems::props::collect);
        app.add_system(Stage::Update, systems::props::dress);
        app.add_system(Stage::Update, systems::hunter::raise);
        app.add_system(Stage::Update, systems::screens::watch_the_keys);
        app.add_system(Stage::Update, systems::screens::drive_the_menus);
        app.add_system(Stage::Update, systems::screens::settle);
        app.add_system(Stage::Update, systems::look::update);
        app.add_system(
            Stage::Update,
            while_in(Screen::Playing, systems::player::update),
        );
        app.add_system(
            Stage::Update,
            while_in(Screen::Playing, systems::hand::update),
        );
        app.add_system(
            Stage::Update,
            while_in(Screen::Playing, systems::hunter::update),
        );
        app.add_system(Stage::Update, systems::hunter::animate);
        app.add_system(Stage::Update, systems::hud::show);
    }
}
