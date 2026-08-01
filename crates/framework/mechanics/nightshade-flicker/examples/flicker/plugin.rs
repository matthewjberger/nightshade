use crate::systems;
use crate::theme;
use nightshade::app::{App, Stage};
use nightshade::plugin::Plugin;

pub struct ExamplePlugin;

impl Plugin for ExamplePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(crate::resources::Street::default());
        app.add_plugin(nightshade_flicker::FlickerPlugin {
            settings: nightshade_flicker::FlickerSettings {
                depth: theme::DEPTH,
                dropout_chance: theme::DROPOUT_CHANCE,
                dropout_scale: theme::DROPOUT_SCALE,
                response: theme::RESPONSE,
                dark_below: theme::DARK_BELOW,
            },
        });
        app.add_plugin(nightshade_camera_controller::CameraControllerPlugin);
        app.add_plugin(nightshade_readout::ReadoutPlugin::titled(theme::TITLE));
        app.add_system(Stage::Startup, systems::setup::stand_it_up);
        app.add_system(Stage::First, systems::play::take_the_orders);
        app.add_system(Stage::Update, systems::play::show_the_street);
    }
}
