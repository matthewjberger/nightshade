use crate::data::Ask;
use crate::resources::HairpinResources;
use crate::systems;
use nightshade::prelude::*;
use nightshade_input::InputPlugin;
use nightshade_platform::prelude::*;

pub struct HairpinPlugin;

impl Plugin for HairpinPlugin {
    fn build(&self, app: &mut App) {
        app.world.res_mut::<Window>().title = "Hairpin".to_string();
        app.insert_resource(HairpinResources::default());
        app.add_plugin(InputPlugin::<Ask>::new());
        app.add_system(Stage::Startup, systems::setup::build_overlays);
        app.add_system(Stage::Startup, systems::setup::initialize);
        app.add_system(
            Stage::Update,
            (
                systems::models::build,
                systems::menus::update,
                systems::drive::update,
                systems::lap::update,
                systems::camera::follow,
            ),
        );
    }
}
