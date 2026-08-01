use crate::systems;
use crate::theme;
use nightshade::app::{App, Stage};
use nightshade::plugin::Plugin;

pub struct ExamplePlugin;

impl Plugin for ExamplePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(crate::resources::Sky::default());
        app.add_plugin(nightshade_daynight::DayNightPlugin {
            settings: nightshade_daynight::DaySettings {
                hours_a_second: theme::HOURS_A_SECOND,
                dawn: theme::DAWN,
                dusk: theme::DUSK,
            },
            hour: theme::START_HOUR,
        });
        app.add_plugin(nightshade_camera_controller::CameraControllerPlugin);
        app.add_plugin(nightshade_readout::ReadoutPlugin::titled(theme::TITLE));
        app.add_system(Stage::Startup, systems::setup::stand_it_up);
        app.add_system(Stage::First, systems::play::take_the_orders);
        app.add_system(Stage::LateUpdate, systems::play::watch_the_sky);
    }
}
