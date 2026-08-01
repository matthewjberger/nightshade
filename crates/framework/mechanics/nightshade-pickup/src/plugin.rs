use crate::systems::frame::gather_the_pickups;
use nightshade::app::{App, Stage};
use nightshade::plugin::Plugin;

pub struct PickupPlugin;

impl Plugin for PickupPlugin {
    fn build(&self, app: &mut App) {
        app.world
            .resource_or_insert_default::<nightshade_plane::PlayPlane>();
        app.add_system(Stage::LateUpdate, gather_the_pickups);
    }
}
