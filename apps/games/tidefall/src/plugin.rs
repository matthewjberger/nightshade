use crate::data::{Ask, Settings};
use crate::resources::{Arguments, Screen, TidefallResources};
use crate::systems;
use nightshade::prelude::*;
use nightshade_config::prelude::ConfigPlugin;
use nightshade_input::InputPlugin;
use nightshade_platform::prelude::*;
use nightshade_state::prelude::*;
use nightshade_ui_config::prelude::ConfigUiPlugin;
use nightshade_ui_menu::prelude::*;

pub struct TidefallPlugin;

impl Plugin for TidefallPlugin {
    fn build(&self, app: &mut App) {
        app.world.res_mut::<Window>().title = "Tidefall".to_string();
        app.insert_resource(TidefallResources::default());
        app.add_plugin(InputPlugin::<Ask>::new());
        app.add_plugin(ConfigPlugin::<Settings>::new().persist("tidefall", "settings.toml"));
        app.add_plugin(ConfigUiPlugin::<Settings>::new());
        let start = if app.world.res::<Arguments>().playing {
            Screen::Playing
        } else {
            Screen::Title
        };
        app.add_plugin(StatePlugin::new(start));
        app.add_plugin(MenuPlugin::<Screen>::new(systems::screens::menu_style()));
        app.add_system(Stage::Startup, systems::setup::initialize);
        app.on_enter(Screen::Playing, systems::screens::enter_the_world);
        app.on_exit(Screen::Playing, systems::screens::leave_the_world);
        app.add_system(Stage::Update, systems::screens::watch_the_keys);
        app.add_system(Stage::Update, systems::screens::drive_the_menus);
        app.add_system(
            Stage::Update,
            while_in(
                Screen::Playing,
                (
                    systems::cockpit::hold_the_window,
                    systems::cockpit::switch_the_lights,
                    systems::orbit::circle,
                    systems::board::change_the_mode,
                    systems::autopilot::steer,
                    systems::flight::fly,
                    systems::walk::walk,
                    systems::talk::speak,
                    systems::hud::measure,
                    systems::hud::paint,
                ),
            ),
        );
    }
}
