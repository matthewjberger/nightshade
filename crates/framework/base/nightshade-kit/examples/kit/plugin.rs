use crate::data::Plan;
use crate::resources::Village;
use crate::systems;
use nightshade::app::{App, Stage};
use nightshade::plugin::Plugin;
use nightshade_config::prelude::ConfigPlugin;
use nightshade_ui_config::prelude::ConfigUiPlugin;

pub struct ExamplePlugin;

impl Plugin for ExamplePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Village::default());
        app.add_plugin(ConfigPlugin::<Plan>::new());
        app.add_plugin(ConfigUiPlugin::<Plan>::new());
        app.add_plugin(nightshade_gltf::GltfPlugin);
        app.add_plugin(nightshade_camera_controller::CameraControllerPlugin);
        app.add_plugin(nightshade_readout::ReadoutPlugin::titled("Kit"));
        app.add_system(Stage::Startup, systems::setup::stand_it_up);
        app.add_system(Stage::Update, systems::village::read_the_plan);
        app.add_system(Stage::Update, systems::village::lay_the_village);
        app.add_system(Stage::Update, systems::village::raise_what_landed);
        app.add_system(Stage::Update, systems::village::write_the_readout);
    }
}
