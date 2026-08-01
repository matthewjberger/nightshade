use crate::resources::DaySettings;
use crate::systems::frame::turn_the_hour;
use nightshade::app::{App, Stage};
use nightshade::plugin::Plugin;
use nightshade_clock::Clock;

#[derive(Default)]
pub struct DayNightPlugin {
    pub settings: DaySettings,
    pub hour: f32,
}

impl Plugin for DayNightPlugin {
    fn needs(&self) -> Vec<nightshade::plugin::Need> {
        nightshade::needs![
            nightshade_platform::contract::time::Time => "PlatformPlugin",
        ]
    }
    fn build(&self, app: &mut App) {
        app.insert_resource(DaySettings {
            hours_a_second: self.settings.hours_a_second,
            dawn: self.settings.dawn,
            dusk: self.settings.dusk,
        });
        app.insert_resource(Clock(self.hour));
        app.add_system(Stage::Update, turn_the_hour);
    }
}
