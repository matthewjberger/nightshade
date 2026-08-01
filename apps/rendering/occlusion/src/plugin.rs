use crate::systems;
use crate::theme;
use nightshade::prelude::*;
use nightshade_platform::prelude::*;

pub struct OcclusionPlugin;

impl Plugin for OcclusionPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(nightshade_readout::ReadoutPlugin::titled(theme::TITLE));
        app.world.res_mut::<Window>().title = theme::TITLE.to_string();
        app.add_system(Stage::Startup, systems::setup::initialize);
    }
}
