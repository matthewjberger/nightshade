use crate::resources::Loading;
use crate::systems;
use nightshade::prelude::{App, Plugin, Stage};

pub struct GltfPlugin;

impl Plugin for GltfPlugin {
    fn needs(&self) -> Vec<nightshade::plugin::Need> {
        nightshade::needs![
            nightshade_render::resources::capture::ScheduledCapture => "GraphicsPlugin",
        ]
    }
    fn build(&self, app: &mut App) {
        app.insert_resource(Loading::default());
        app.add_system(Stage::Update, systems::loading::collect_the_models);
    }
}
