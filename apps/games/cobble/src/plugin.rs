use crate::systems;
use blocker_screen::data::screen::Screen;
use nightshade::prelude::*;
use nightshade_state::prelude::*;

pub struct CobblePlugin;

impl Plugin for CobblePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(nightshade_random::Random::seeded(
            blocker_screen::theme::RANDOM_SEED,
        ));
        app.add_plugin(StatePlugin::new(Screen::Playing));
        app.add_plugin(nightshade_health::HealthPlugin);
        app.add_plugin(blocker_blocks::plugin::BlocksPlugin);
        app.add_plugin(blocker_pocket::plugin::PocketPlugin);
        app.add_plugin(blocker_weather::plugin::WeatherPlugin);
        app.add_plugin(blocker_player::plugin::PlayerPlugin);
        app.add_plugin(blocker_world::plugin::WorldPlugin);

        app.add_system(
            Stage::Startup,
            (systems::setup::raise_the_blocks, systems::setup::initialize),
        );
        app.startup.order(
            std::any::type_name_of_val(&systems::setup::initialize),
            std::any::type_name_of_val(&blocker_player::systems::arrive::place_player),
        );
    }
}
