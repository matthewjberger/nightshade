use crate::ecs::TemplateResources;
use crate::systems;
use nightshade::prelude::*;

/// The game plugin: sets the window title, inserts the app resources, and
/// registers the startup and per-frame systems. All behavior lives in
/// `src/systems/`; grow your game by adding systems there and registering
/// them here.
pub struct TemplatePlugin;

impl Plugin for TemplatePlugin {
    fn build(&self, app: &mut App) {
        app.world.res_mut::<Window>().title = "Template".to_string();
        app.insert_resource(TemplateResources::default());
        app.add_system(Stage::Startup, systems::setup::initialize);
        app.add_system(Stage::Update, systems::example::tick);
    }
}
