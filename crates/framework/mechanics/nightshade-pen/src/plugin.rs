use crate::systems::frame::keep_them_inside;
use nightshade::app::{App, Stage};
use nightshade::plugin::Plugin;

pub struct PenPlugin;

impl Plugin for PenPlugin {
    fn build(&self, app: &mut App) {
        app.world
            .resource_or_insert_default::<nightshade_plane::PlayPlane>();
        app.add_system(Stage::LateUpdate, keep_them_inside);
    }
}
