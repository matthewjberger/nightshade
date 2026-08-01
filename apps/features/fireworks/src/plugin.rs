use crate::resources::{Interface, Launcher, Show, Toggles};
use crate::systems;
use nightshade::prelude::*;
use nightshade_platform::prelude::*;

pub struct FireworksPlugin;

impl Plugin for FireworksPlugin {
    fn build(&self, app: &mut App) {
        app.world.res_mut::<Window>().title = "Fireworks".to_string();
        app.insert_resource(Launcher::default());
        app.insert_resource(Toggles::default());
        app.insert_resource(Show::default());
        app.insert_resource(Interface::default());
        app.add_system(Stage::Startup, systems::setup::initialize);
        app.add_system(
            Stage::Update,
            (
                systems::interface::poll_ui_events,
                systems::interface::update_ui_state,
                systems::launch::advance_show,
                systems::shells::advance_shells,
                systems::effects::expire_effects,
                systems::input::watch_the_keys,
            ),
        );
    }
}
