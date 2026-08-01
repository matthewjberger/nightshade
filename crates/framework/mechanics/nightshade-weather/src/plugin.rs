use crate::resources::{Weather, WeatherSettings};
use crate::systems::frame::turn_the_weather;
use nightshade::app::{App, Stage};
use nightshade::plugin::Plugin;

#[derive(Default)]
pub struct WeatherPlugin {
    pub settings: WeatherSettings,
}

impl Plugin for WeatherPlugin {
    fn needs(&self) -> Vec<nightshade::plugin::Need> {
        nightshade::needs![
            nightshade_platform::contract::time::Time => "PlatformPlugin",
        ]
    }
    fn build(&self, app: &mut App) {
        app.insert_resource(WeatherSettings {
            shortest: self.settings.shortest,
            longest: self.settings.longest,
            chance_of_rain: self.settings.chance_of_rain,
            dry_rate: self.settings.dry_rate,
        });
        app.insert_resource(Weather::default());
        app.world
            .resource_or_insert_default::<nightshade_random::Random>();
        app.add_system(Stage::Update, turn_the_weather);
    }
}
