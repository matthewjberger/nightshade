use nightshade::app::{App, Stage};
use nightshade::plugin::Plugin;

pub struct DecalsPlugin;

impl Plugin for DecalsPlugin {
    fn build(&self, app: &mut App) {
        app.world
            .resource_or_insert_default::<crate::contract::DecalFrame>();
        app.add_system(Stage::Render, crate::systems::gather_decals_system);
    }
}
