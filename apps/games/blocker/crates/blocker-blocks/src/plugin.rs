use crate::resources::{Art, Sheet};
use crate::systems;
use nightshade::prelude::*;

pub struct BlocksPlugin;

impl Plugin for BlocksPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Art::default());
        app.insert_resource(Sheet::default());
        app.add_system(Stage::Update, systems::textures::repaint_the_blocks);
    }
}
