use crate::systems::model::arrive;
use crate::systems::setup::initialize;
use nightshade::app::{App, Stage};
use nightshade::plugin::Plugin;
use nightshade_readout::ReadoutPlugin;

pub struct PedestalPlugin {
    pub title: &'static str,
}

impl PedestalPlugin {
    pub fn titled(title: &'static str) -> Self {
        Self { title }
    }
}

impl Plugin for PedestalPlugin {
    fn needs(&self) -> Vec<nightshade::plugin::Need> {
        nightshade::needs![
            nightshade_platform::contract::window::Window => "PlatformPlugin",
            nightshade_camera::resources::ActiveCamera => "CameraPlugin",
        ]
    }
    fn build(&self, app: &mut App) {
        app.world
            .resource_or_insert_default::<crate::resources::PendingModel>();
        app.add_plugin(ReadoutPlugin::titled(self.title));
        app.add_system(Stage::Startup, initialize);
        app.add_system(Stage::Update, arrive);
    }
}
