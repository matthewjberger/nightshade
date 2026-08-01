use crate::systems::frame::stand_on_the_ground;
use nightshade::app::{App, Stage};
use nightshade::plugin::Plugin;
use nightshade_ground::Ground;

pub struct GroundedPlugin;

impl Plugin for GroundedPlugin {
    fn build(&self, app: &mut App) {
        app.world.resource_or_insert_default::<Ground>();
        app.add_system(Stage::LateUpdate, stand_on_the_ground);
    }
}
