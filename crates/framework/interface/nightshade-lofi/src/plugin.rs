use crate::data::LofiLook;
use crate::resources::LofiSettings;
use crate::systems::frame::apply_the_look;
use nightshade::app::{App, Stage};
use nightshade::plugin::Plugin;

#[derive(Default)]
pub struct LofiPlugin {
    pub look: LofiLook,
}

impl Plugin for LofiPlugin {
    fn needs(&self) -> Vec<nightshade::plugin::Need> {
        nightshade::needs![
            nightshade_render::data::config::RenderSettings => "GraphicsPlugin",
        ]
    }
    fn build(&self, app: &mut App) {
        app.insert_resource(LofiSettings {
            look: self.look,
            waiting: true,
        });
        app.add_system(Stage::Update, apply_the_look);
    }
}
