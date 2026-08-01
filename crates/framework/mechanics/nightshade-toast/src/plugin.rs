use crate::resources::{Toast, ToastSettings};
use crate::systems::frame::fade_the_toast;
use nightshade::app::{App, Stage};
use nightshade::plugin::Plugin;

#[derive(Default)]
pub struct ToastPlugin {
    pub settings: ToastSettings,
}

impl Plugin for ToastPlugin {
    fn needs(&self) -> Vec<nightshade::plugin::Need> {
        nightshade::needs![
            nightshade_platform::contract::time::Time => "PlatformPlugin",
        ]
    }
    fn build(&self, app: &mut App) {
        app.insert_resource(ToastSettings {
            showing: self.settings.showing,
            fading: self.settings.fading,
        });
        app.insert_resource(Toast::default());
        app.add_system(Stage::LateUpdate, fade_the_toast);
    }
}
