use crate::resources::RoundState;
use crate::systems;
use nightshade::prelude::*;

pub struct BreakoutPlugin;

impl Plugin for BreakoutPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(RoundState::default());
        app.add_plugin(nightshade_input::InputPlugin::<crate::data::Ask>::new());
        app.add_system(Stage::Startup, systems::setup::initialize);
        app.add_system(
            Stage::Update,
            (
                systems::round::rebuild,
                systems::paddle::drive,
                systems::ball::advance,
                systems::round::settle,
                systems::readout::show,
            ),
        );
    }
}
