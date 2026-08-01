use crate::systems;
use crate::theme;
use nightshade::app::{App, Stage};
use nightshade::plugin::Plugin;

pub struct ExamplePlugin;

impl Plugin for ExamplePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(nightshade_wander::WanderPlugin {
            settings: nightshade_wander::WanderSettings {
                reach: theme::ROAM,
                speed: theme::SPEED,
                arrival: theme::ARRIVAL,
                rest_low: theme::REST_LOW,
                rest_high: theme::REST_HIGH,
            },
        });
        app.add_plugin(nightshade_camera_controller::CameraControllerPlugin);
        app.add_plugin(nightshade_readout::ReadoutPlugin::titled(theme::TITLE));
        app.add_system(Stage::Startup, systems::setup::stand_it_up);
        app.add_system(Stage::First, systems::play::take_the_orders);
        app.add_system(Stage::Update, systems::play::count_the_resting);
    }
}
