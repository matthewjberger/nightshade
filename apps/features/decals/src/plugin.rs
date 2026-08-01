use crate::resources::DecalsResources;
use crate::systems;
use nightshade::prelude::*;
use nightshade_platform::prelude::*;

pub struct DecalsDemoPlugin;

impl Plugin for DecalsDemoPlugin {
    fn build(&self, app: &mut App) {
        app.world.res_mut::<Window>().title = "Decals Demo".to_string();
        app.insert_resource(DecalsResources::default());
        app.add_system(Stage::Startup, systems::setup::initialize);
        app.add_system(
            Stage::Update,
            (
                systems::panel::poll,
                systems::hover::track,
                systems::hover::update_preview,
                systems::input::handle_events,
                systems::decals::apply_clear,
                systems::panel::refresh_text,
            ),
        );
    }
}
