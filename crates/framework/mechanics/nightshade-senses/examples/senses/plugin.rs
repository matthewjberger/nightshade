use crate::systems;
use crate::theme;
use nightshade::app::{App, Stage};
use nightshade::plugin::Plugin;

pub struct ExamplePlugin;

impl Plugin for ExamplePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(crate::resources::Field::default());
        app.add_plugin(nightshade_senses::SensesPlugin {
            settings: nightshade_senses::SensesSettings {
                plane: nightshade_plane::Plane::Ground,
                crouch_scale: theme::CROUCH_SCALE,
                moving_scale: theme::MOVING_SCALE,
                half_angle: theme::HALF_ANGLE,
                noise_gain: theme::NOISE_GAIN,
                calm: theme::CALM,
                alarmed_above: theme::ALARMED_ABOVE,
            },
        });
        app.add_plugin(nightshade_camera_controller::CameraControllerPlugin);
        app.add_plugin(nightshade_readout::ReadoutPlugin::titled(theme::TITLE));
        app.add_system(Stage::Startup, systems::setup::stand_it_up);
        app.add_system(Stage::First, systems::play::take_the_orders);
        app.add_system(Stage::Update, systems::play::turn_the_watchers);
        app.add_system(Stage::FrameUpdate, systems::play::paint_the_watchers);
        app.add_system(Stage::FrameUpdate, systems::play::show_the_field);
    }
}
