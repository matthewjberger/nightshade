use crate::resources::{WavePlan, WaveProgress};
use crate::systems::frame::send_the_wave;
use nightshade::app::{App, Stage};
use nightshade::plugin::Plugin;

pub struct WavesPlugin;

impl Plugin for WavesPlugin {
    fn needs(&self) -> Vec<nightshade::plugin::Need> {
        nightshade::needs![
            nightshade_platform::contract::time::Time => "PlatformPlugin",
        ]
    }
    fn build(&self, app: &mut App) {
        app.world.resource_or_insert_default::<WavePlan>();
        app.insert_resource(WaveProgress::default());
        app.add_system(Stage::Update, send_the_wave);
    }
}
