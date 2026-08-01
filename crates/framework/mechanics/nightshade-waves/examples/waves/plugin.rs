use crate::systems;
use crate::theme;
use nightshade::app::{App, Stage};
use nightshade::plugin::Plugin;

pub struct ExamplePlugin;

impl Plugin for ExamplePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(crate::resources::Gate::default());
        app.add_plugin(nightshade_camera_controller::CameraControllerPlugin);
        app.add_plugin(nightshade_readout::ReadoutPlugin::titled(theme::TITLE));
        app.add_plugin(nightshade_waves::WavesPlugin);
        app.add_system(Stage::Startup, systems::setup::open_the_gate);
        app.add_system(Stage::First, systems::play::take_the_orders);
        app.add_system(Stage::Update, systems::play::meet_the_arrivals);
        app.add_system(Stage::Update, systems::play::count_the_cleared);
        app.add_system(Stage::Update, systems::play::walk_the_lane);
        app.add_system(Stage::Update, systems::play::show_the_wave);
    }
}
