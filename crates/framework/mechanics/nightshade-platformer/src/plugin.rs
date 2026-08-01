use crate::resources::PlatformerSettings;
use crate::systems::frame::spend_the_mercy;
use nightshade::app::{App, Stage};
use nightshade::plugin::Plugin;

#[derive(Default)]
pub struct PlatformerPlugin {
    pub settings: PlatformerSettings,
}

impl Plugin for PlatformerPlugin {
    fn needs(&self) -> Vec<nightshade::plugin::Need> {
        nightshade::needs![
            nightshade_platform::contract::time::Time => "PlatformPlugin",
        ]
    }
    fn build(&self, app: &mut App) {
        app.insert_resource(PlatformerSettings {
            coyote: self.settings.coyote,
            buffer: self.settings.buffer,
        });
        app.add_system(Stage::First, spend_the_mercy);
    }
}
