use crate::resources::Opening;
use nightshade::app::{App, Stage};
use nightshade::plugin::Plugin;

pub struct OpeningPlugin {
    pub opening: Opening,
}

impl Plugin for OpeningPlugin {
    fn needs(&self) -> Vec<nightshade::plugin::Need> {
        nightshade::needs![
            nightshade_platform::contract::window::Window => "PlatformPlugin",
        ]
    }
    fn build(&self, app: &mut App) {
        app.insert_resource(self.opening);
        app.add_system(Stage::Startup, crate::systems::opening::name_the_window);
    }
}
