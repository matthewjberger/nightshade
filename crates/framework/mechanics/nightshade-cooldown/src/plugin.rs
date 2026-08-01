use crate::systems::frame::spend_the_cooldowns;
use nightshade::app::{App, Stage};
use nightshade::plugin::Plugin;

pub struct CooldownPlugin;

impl Plugin for CooldownPlugin {
    fn needs(&self) -> Vec<nightshade::plugin::Need> {
        nightshade::needs![
            nightshade_platform::contract::time::Time => "PlatformPlugin",
        ]
    }
    fn build(&self, app: &mut App) {
        app.add_system(Stage::First, spend_the_cooldowns);
    }
}
