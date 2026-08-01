use crate::resources::{Arrival, Bench, Digging, Draft, Effects, Generation};
use crate::systems;
use nightshade::prelude::*;
use nightshade_text::TextPlugin;
use nightshade_voxels::VoxelsPlugin as BlockWorldPlugin;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Generation::default());
        app.insert_resource(Bench::default());
        app.insert_resource(Draft::default());
        app.insert_resource(Effects::default());
        app.insert_resource(Arrival::default());
        app.add_plugin(BlockWorldPlugin::<Digging>::default());
        app.add_plugin(TextPlugin);
        app.add_system(
            Stage::Update,
            (
                systems::fresh::clear_the_ground,
                systems::effects::follow_the_blocks,
            ),
        );
    }
}
