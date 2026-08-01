use crate::resources::TopDownSettings;
use crate::systems::frame::walk;
use nightshade::app::{App, Stage};
use nightshade::plugin::Plugin;

#[derive(Default)]
pub struct TopDownPlugin {
    pub settings: TopDownSettings,
}

impl Plugin for TopDownPlugin {
    fn needs(&self) -> Vec<nightshade::plugin::Need> {
        nightshade::needs![
            nightshade_platform::contract::time::Time => "PlatformPlugin",
            nightshade_input::contract::Actions<nightshade_input::contract::EngineAction> => "InputPlugin<EngineAction>",
        ]
    }
    fn build(&self, app: &mut App) {
        app.insert_resource(TopDownSettings {
            speed: self.settings.speed,
        });
        app.add_system(Stage::Update, walk);
    }
}
