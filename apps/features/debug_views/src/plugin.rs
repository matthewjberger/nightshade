use crate::resources::DebugViewsResources;
use crate::systems;
use crate::theme;
use nightshade::prelude::*;
use nightshade_platform::prelude::*;

pub struct DebugViewsPlugin;

impl Plugin for DebugViewsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(nightshade_readout::ReadoutPlugin::titled(theme::TITLE));
        app.world.res_mut::<Window>().title = theme::TITLE.to_string();
        app.insert_resource(DebugViewsResources::default());
        app.add_system(
            Stage::Startup,
            (systems::setup::initialize, systems::setup::build_panel),
        );
        app.add_system(
            Stage::Update,
            (
                systems::setup::stand_the_scene,
                systems::panel::poll,
                systems::panel::advance,
                systems::panel::refresh_label,
                systems::panel::drain_keyboard,
            ),
        );
    }
}
