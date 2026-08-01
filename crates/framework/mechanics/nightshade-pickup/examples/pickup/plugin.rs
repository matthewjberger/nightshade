use crate::systems;
use crate::theme;
use nightshade::app::{App, Stage};
use nightshade::plugin::Plugin;

pub struct ExamplePlugin;

impl Plugin for ExamplePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(crate::resources::Sweep::default());
        app.insert_resource(nightshade_inventory::StackLimit::default());
        app.add_plugin(nightshade_camera_controller::CameraControllerPlugin);
        app.add_plugin(nightshade_readout::ReadoutPlugin::titled(theme::TITLE));
        app.add_plugin(nightshade_pickup::PickupPlugin);
        app.add_system(Stage::Startup, systems::setup::scatter_the_coins);
        app.add_system(Stage::First, systems::play::take_the_orders);
        app.add_system(Stage::Last, systems::play::take_the_coins);
        app.add_system(Stage::Last, systems::play::show_the_purse);
    }
}
