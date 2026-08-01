use crate::data;
use crate::data::Ask;
use crate::resources::NeonStaticResources;
use crate::systems;
use nightshade::prelude::*;
use nightshade_input::InputPlugin;
use nightshade_platform::prelude::*;

pub struct NeonStaticPlugin;

impl Plugin for NeonStaticPlugin {
    fn build(&self, app: &mut App) {
        app.world.res_mut::<Window>().title = data::TITLE.to_string();
        app.insert_resource(NeonStaticResources::default());
        app.add_plugin(InputPlugin::<Ask>::new());
        app.add_system(Stage::Startup, systems::hud::build_overlay);
        app.add_system(Stage::Startup, systems::setup::initialize);
        app.add_system(
            Stage::Update,
            (
                systems::player::drive,
                systems::enemies::hunt,
                systems::bullets::advance,
                systems::shells::settle,
                systems::floor::settle,
                systems::camera::follow,
                systems::hud::show,
            ),
        );
    }
}
