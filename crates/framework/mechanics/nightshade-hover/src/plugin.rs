use crate::resources::HoverSettings;
use crate::systems::frame::hold_them_up;
use nightshade::app::{App, Stage};
use nightshade::plugin::Plugin;

#[derive(Default)]
pub struct HoverPlugin {
    pub settings: HoverSettings,
}

impl Plugin for HoverPlugin {
    fn needs(&self) -> Vec<nightshade::plugin::Need> {
        nightshade::needs![
            nightshade_platform::contract::time::Time => "PlatformPlugin",
        ]
    }
    fn build(&self, app: &mut App) {
        app.insert_resource(HoverSettings {
            rate: self.settings.rate,
        });
        app.add_system(Stage::LateUpdate, hold_them_up);
    }
}
