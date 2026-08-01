use crate::resources::ShakeSettings;
use crate::systems::frame::settle_the_shake;
use nightshade::app::{App, Stage};
use nightshade::plugin::Plugin;

#[derive(Default)]
pub struct ShakePlugin {
    pub settings: ShakeSettings,
}

impl Plugin for ShakePlugin {
    fn needs(&self) -> Vec<nightshade::plugin::Need> {
        nightshade::needs![
            nightshade_platform::contract::time::Time => "PlatformPlugin",
        ]
    }
    fn build(&self, app: &mut App) {
        app.insert_resource(ShakeSettings {
            decay: self.settings.decay,
            reach: self.settings.reach,
            turn: self.settings.turn,
            ceiling: self.settings.ceiling,
        });
        app.world
            .resource_or_insert_default::<nightshade_random::Random>();
        app.add_system(Stage::LateUpdate, settle_the_shake);
    }
}
