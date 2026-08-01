use crate::data;
use crate::data::Ask;
use crate::resources::PortalsResources;
use crate::resources::Screen;
use crate::systems;
use nightshade::prelude::*;
use nightshade_input::InputPlugin;
use nightshade_platform::prelude::*;
use nightshade_portal::prelude::PortalPlugin;
use nightshade_state::prelude::*;
use nightshade_ui_menu::prelude::*;

pub struct PortalsPlugin;

impl Plugin for PortalsPlugin {
    fn build(&self, app: &mut App) {
        app.world.res_mut::<Window>().title = data::TITLE.to_string();
        app.insert_resource(PortalsResources::default());
        app.add_plugin(StatePlugin::new(Screen::Playing));
        app.add_plugin(InputPlugin::<Ask>::new());
        app.add_plugin(MenuPlugin::<Screen>::new(systems::menu::menu_style()));
        app.on_enter(Screen::Playing, systems::menu::hold_the_cursor);
        app.on_enter(Screen::Paused, systems::menu::release_the_cursor);
        app.add_system(Stage::Startup, systems::setup::build_overlay);
        app.add_system(Stage::Startup, systems::setup::initialize);
        app.add_system(Stage::Update, systems::menu::update);
        app.add_system(
            Stage::Update,
            while_in(Screen::Playing, systems::player::look),
        );
        app.add_system(
            Stage::Update,
            while_in(Screen::Playing, systems::player::walk),
        );
        app.add_system(
            Stage::Update,
            while_in(Screen::Playing, systems::travel::update),
        );
        app.add_system(Stage::Update, systems::player::place);
        app.add_system(Stage::Update, while_in(Screen::Playing, systems::gun::fire));
        app.add_system(Stage::Update, systems::sparks::update);
        app.add_system(Stage::Update, systems::hud::show);
        app.add_plugin(PortalPlugin);
    }
}
