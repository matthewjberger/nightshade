use crate::resources::{Behavior, FoxModel, Ground, Interface, Names, Spawns, Trees, Viewpoint};
use crate::systems;
use nightshade::prelude::*;
use nightshade_platform::prelude::*;

pub struct NavMeshDemoPlugin;

impl Plugin for NavMeshDemoPlugin {
    fn build(&self, app: &mut App) {
        app.world.res_mut::<Window>().title = "NavMesh Pathfinding".to_string();
        app.insert_resource(Ground::default());
        app.insert_resource(Trees::default());
        app.insert_resource(Names::default());
        app.insert_resource(Behavior::default());
        app.insert_resource(FoxModel::default());
        app.insert_resource(Spawns::default());
        app.insert_resource(Viewpoint::default());
        app.insert_resource(Interface::default());
        app.add_system(Stage::Startup, systems::setup::initialize);
        app.add_system(
            Stage::Update,
            (
                systems::foxes::stand_the_foxes,
                systems::camera::gate_orbit_camera,
                systems::interface::poll_ui_events,
                systems::input::handle_mouse_input,
                systems::wander::update_wander_mode,
                systems::foxes::sync_foxes_to_agents,
                systems::foxes::update_fox_animations,
                systems::camera::update_follow_camera,
                systems::interface::update_ui_labels,
            ),
        );
    }
}
