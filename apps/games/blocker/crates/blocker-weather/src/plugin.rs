use crate::data::settings::Settings;
use crate::resources::{Falling, Overhead, Season, Settling, Sky};
use crate::systems;
use nightshade::prelude::*;
use nightshade_config::prelude::ConfigPlugin;

pub struct WeatherPlugin;

impl Plugin for WeatherPlugin {
    fn needs(&self) -> Vec<nightshade::plugin::Need> {
        nightshade::needs![nightshade_random::Random => "the app plugin"]
    }

    fn build(&self, app: &mut App) {
        app.add_plugin(ConfigPlugin::<Settings>::new().persist("blocker", "weather.toml"));
        app.insert_resource(Sky::default());
        app.insert_resource(Season::default());
        app.insert_resource(Overhead::default());
        app.insert_resource(Falling::default());
        app.insert_resource(Settling::default());
        app.add_system(
            Stage::Update,
            (
                systems::fresh::clear_the_sky,
                systems::sky::paint_the_sky,
                systems::weather::turn_the_weather,
                systems::cover::settle_the_snow,
            ),
        );
    }
}
