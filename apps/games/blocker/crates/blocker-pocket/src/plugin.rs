use crate::resources::{Chests, Inventory, OpenChest};
use crate::systems;
use nightshade::prelude::*;

pub struct PocketPlugin;

impl Plugin for PocketPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Inventory::default());
        app.insert_resource(Chests::default());
        app.insert_resource(OpenChest::default());
        app.add_system(Stage::Update, systems::fresh::empty_the_pockets);
    }
}
