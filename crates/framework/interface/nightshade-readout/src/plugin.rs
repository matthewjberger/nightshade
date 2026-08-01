use crate::resources::Readout;
use crate::systems::title::show_the_readout;
use nightshade::app::{App, Stage};
use nightshade::plugin::Plugin;

pub struct ReadoutPlugin {
    pub title: &'static str,
}

impl ReadoutPlugin {
    pub fn titled(title: &'static str) -> Self {
        Self { title }
    }
}

impl Plugin for ReadoutPlugin {
    fn needs(&self) -> Vec<nightshade::plugin::Need> {
        nightshade::needs![
            nightshade_platform::contract::time::Time => "PlatformPlugin",
            nightshade_platform::contract::window::Window => "PlatformPlugin",
        ]
    }
    fn build(&self, app: &mut App) {
        app.insert_resource(Readout {
            title: self.title.to_string(),
            notes: Vec::new(),
        });
        app.add_system(Stage::Update, show_the_readout);
    }
}
