use crate::contract::OrreryCommands;
use crate::data::Ask;
use crate::resources::{Arguments, OrreryResources, Screen};
use crate::systems;
use nightshade::prelude::*;
use nightshade_input::InputPlugin;
use nightshade_platform::prelude::*;
use nightshade_state::prelude::*;
use nightshade_ui_menu::prelude::*;

pub struct OrreryPlugin;

impl Plugin for OrreryPlugin {
    fn build(&self, app: &mut App) {
        app.world.res_mut::<Window>().title = "Orrery".to_string();
        app.insert_resource(OrreryResources::default());
        app.insert_resource(OrreryCommands::default());
        let opening = if app.world.res::<Arguments>().fly {
            Screen::Playing
        } else {
            Screen::Title
        };
        app.add_plugin(StatePlugin::new(opening));
        app.add_plugin(InputPlugin::<Ask>::new());
        app.add_plugin(MenuPlugin::<Screen>::new(systems::hud::menu_style()));
        app.add_system(Stage::Startup, systems::hud::build);
        app.add_system(Stage::Startup, systems::setup::initialize);
        app.add_system(
            Stage::Update,
            (
                systems::menus::read_state,
                systems::menus::drive,
                systems::menus::show_screen,
                systems::menus::hold_cursor,
                while_in_any(
                    [Screen::Title, Screen::Playing],
                    (
                        systems::orbits::advance,
                        systems::rubble::tumble,
                        systems::flight::fly,
                        systems::thrusters::blow,
                        systems::camera::place,
                    ),
                ),
                while_in(
                    Screen::Playing,
                    (
                        systems::walk::walk,
                        systems::mode::apply,
                        systems::beacons::touch,
                        systems::hud::show,
                    ),
                ),
                systems::menus::apply_request,
            ),
        );
    }
}
