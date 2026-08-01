use crate::resources::Pending;
use crate::systems;
use crate::theme;
use nightshade::prelude::*;
use nightshade_platform::prelude::*;

pub struct TracedReflectionsPlugin;

impl Plugin for TracedReflectionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(nightshade_readout::ReadoutPlugin::titled(theme::TITLE));
        app.world.res_mut::<Window>().title = theme::TITLE.to_string();
        app.insert_resource(Pending::default());
        app.add_system(Stage::Startup, systems::setup::initialize);
        app.add_system(Stage::Update, (systems::setup::stand_the_scene,));
    }
}
