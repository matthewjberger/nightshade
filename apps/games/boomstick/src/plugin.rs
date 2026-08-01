use crate::contract::FireQueue;
use crate::resources::{Arguments, BoomstickResources, Screen};
use crate::systems;
use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_state::prelude::*;
use nightshade_ui_menu::prelude::*;

pub struct BoomstickPlugin;

impl Plugin for BoomstickPlugin {
    fn build(&self, app: &mut App) {
        app.world.res_mut::<Window>().title = "Boomstick".to_string();
        app.insert_resource(BoomstickResources::default());
        app.insert_resource(FireQueue::default());
        let opening = if app.world.res::<Arguments>().level.is_some() {
            Screen::Level
        } else {
            Screen::Title
        };
        app.add_plugin(StatePlugin::new(opening));
        app.add_plugin(nightshade_input::InputPlugin::<crate::data::Ask>::new());
        app.add_plugin(MenuPlugin::<Screen>::new(systems::screens::style()));

        app.add_system(Stage::Startup, systems::setup::initialize);
        app.add_system(Stage::Startup, systems::setup::load_particles);
        app.add_system(Stage::Startup, systems::setup::request_models);

        app.add_system(Stage::Update, systems::pace::steady);
        app.add_system(Stage::Update, systems::models::collect);
        app.add_system(Stage::Update, systems::player::grip);
        app.add_system(Stage::Update, systems::screens::watch_the_keys);
        app.add_system(Stage::Update, systems::screens::drive_the_menus);

        app.on_exit(Screen::Tally, systems::screens::step_the_floor);
        app.on_enter(Screen::Title, systems::screens::open_title);
        app.on_enter(Screen::Title, systems::screens::hide_hud);
        app.on_enter(Screen::Level, systems::mission::request);
        app.on_enter(Screen::Level, systems::screens::show_hud);
        app.on_exit(Screen::Level, systems::screens::hide_hud);

        app.add_system(
            Stage::Update,
            while_in(
                Screen::Level,
                (
                    systems::mission::build,
                    systems::populate::models,
                    systems::populate::loot,
                    systems::look::update,
                    systems::player::update,
                    systems::weapon::update,
                    systems::viewmodel::carry,
                    systems::viewmodel::hold,
                ),
            ),
        );
        app.add_system(
            Stage::Update,
            while_in(
                Screen::Level,
                (
                    systems::shot::resolve,
                    systems::projectiles::player_shots,
                    systems::projectiles::enemy_shots,
                    systems::blast::resolve,
                    systems::enemies::settle,
                    systems::enemies::update,
                ),
            ),
        );
        app.add_system(
            Stage::Update,
            while_in(
                Screen::Level,
                (
                    systems::pickups::update,
                    systems::doors::update,
                    systems::triggers::ambush,
                    systems::triggers::hazards,
                    systems::triggers::exit,
                    systems::mission::track,
                    systems::mission::finish,
                ),
            ),
        );

        app.add_system(
            Stage::LateUpdate,
            while_in(Screen::Level, systems::hud::show),
        );
        app.add_system(Stage::LateUpdate, systems::screens::write_the_pages);
    }
}
