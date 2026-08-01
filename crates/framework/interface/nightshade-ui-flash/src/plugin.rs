use crate::resources::{Flash, FlashHandles, FlashShown};
use crate::systems::frame::fade_the_flash;
use nightshade::app::{App, Stage};
use nightshade::plugin::Plugin;

#[derive(Default)]
pub struct FlashPlugin;

impl Plugin for FlashPlugin {
    fn needs(&self) -> Vec<nightshade::plugin::Need> {
        nightshade::needs![
            nightshade_platform::contract::time::Time => "PlatformPlugin",
        ]
    }
    fn build(&self, app: &mut App) {
        app.insert_resource(Flash::default());
        app.insert_resource(FlashHandles::default());
        app.insert_resource(FlashShown::default());
        app.add_system(Stage::Update, fade_the_flash);
    }
}
