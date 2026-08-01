use crate::systems::frame::take_the_damage;
use nightshade::app::{App, Stage};
use nightshade::plugin::Plugin;

pub struct HealthPlugin;

impl Plugin for HealthPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(Stage::LateUpdate, take_the_damage);
    }
}
