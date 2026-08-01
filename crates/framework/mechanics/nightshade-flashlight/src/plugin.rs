use crate::resources::FlashlightSettings;
use crate::systems::frame::drain_the_cell;
use nightshade::app::{App, Stage};
use nightshade::plugin::Plugin;

#[derive(Default)]
pub struct FlashlightPlugin {
    pub settings: FlashlightSettings,
}

impl Plugin for FlashlightPlugin {
    fn needs(&self) -> Vec<nightshade::plugin::Need> {
        nightshade::needs![
            nightshade_platform::contract::time::Time => "PlatformPlugin",
        ]
    }
    fn build(&self, app: &mut App) {
        app.insert_resource(FlashlightSettings {
            full: self.settings.full,
            low: self.settings.low,
            brownout: self.settings.brownout,
        });
        app.add_system(Stage::LateUpdate, drain_the_cell);
    }
}
