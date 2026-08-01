use crate::resources::{Journal, Waypoint};
use crate::systems;
use nightshade::prelude::*;

pub struct QuestsPlugin;

impl Plugin for QuestsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Journal::default());
        app.insert_resource(Waypoint::default());
        app.add_system(Stage::Update, systems::fresh::clear_the_journal);
    }
}
