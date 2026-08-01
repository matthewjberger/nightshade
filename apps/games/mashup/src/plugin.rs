use crate::data;
use crate::resources::MashupResources;
use crate::resources::Screen;
use crate::systems;
use nightshade::prelude::*;
use nightshade_input::InputPlugin;
use nightshade_platform::prelude::*;
use nightshade_state::prelude::*;
use nightshade_ui_menu::prelude::*;

pub struct MashupPlugin;

impl Plugin for MashupPlugin {
    fn build(&self, app: &mut App) {
        app.world.res_mut::<Window>().title = data::TITLE.to_string();
        app.insert_resource(MashupResources::default());
        app.add_plugin(StatePlugin::new(Screen::Playing));
        app.add_plugin(InputPlugin::<data::Ask>::new());
        app.add_plugin(MenuPlugin::<Screen>::new(systems::menu::menu_style()));
        app.on_enter(Screen::Playing, systems::menu::hold_the_cursor);
        app.on_enter(Screen::Paused, systems::menu::release_the_cursor);
        app.add_system(Stage::Startup, systems::setup::build_overlay);
        app.add_system(Stage::Startup, systems::setup::initialize);
        app.add_system(Stage::Update, systems::menu::update);
        app.add_system(
            Stage::Update,
            while_in(Screen::Playing, systems::meadow::turn_camera),
        );
        app.add_system(
            Stage::Update,
            while_in(Screen::Playing, systems::meadow::walk),
        );
        app.add_system(
            Stage::Update,
            while_in(Screen::Playing, systems::meadow::wander),
        );
        app.add_system(
            Stage::Update,
            while_in(Screen::Playing, systems::meadow::place),
        );
        app.add_system(
            Stage::Update,
            while_in(Screen::Playing, systems::quarry::look),
        );
        app.add_system(
            Stage::Update,
            while_in(Screen::Playing, systems::quarry::walk),
        );
        app.add_system(
            Stage::Update,
            while_in(Screen::Playing, systems::crossing::update),
        );
        app.add_system(Stage::Update, systems::crossing::place_camera);
        app.add_system(Stage::Update, systems::meadow::turn_uprights);
        app.add_system(Stage::Update, systems::portal_view::update);
        app.add_system(Stage::Update, systems::sparks::update);
        app.add_system(Stage::Update, systems::hud::show);
    }
}
