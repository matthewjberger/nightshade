use crate::resources::FlickerSettings;
use crate::systems::frame::flicker_the_lights;
use nightshade::app::{App, Stage};
use nightshade::plugin::Plugin;

#[derive(Default)]
pub struct FlickerPlugin {
    pub settings: FlickerSettings,
}

impl Plugin for FlickerPlugin {
    fn needs(&self) -> Vec<nightshade::plugin::Need> {
        nightshade::needs![
            nightshade_platform::contract::time::Time => "PlatformPlugin",
        ]
    }
    fn build(&self, app: &mut App) {
        app.insert_resource(FlickerSettings {
            depth: self.settings.depth,
            dropout_chance: self.settings.dropout_chance,
            dropout_scale: self.settings.dropout_scale,
            response: self.settings.response,
            dark_below: self.settings.dark_below,
        });
        app.world
            .resource_or_insert_default::<nightshade_random::Random>();
        app.add_system(Stage::LateUpdate, flicker_the_lights);
    }
}
