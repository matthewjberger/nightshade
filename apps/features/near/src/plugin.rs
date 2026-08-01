use crate::systems;
use crate::theme;
use nightshade::app::{App, Stage};
use nightshade::plugin::Plugin;

pub struct NearExamplePlugin;

impl Plugin for NearExamplePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(crate::resources::Field::default());
        app.add_plugin(nightshade_camera_controller::CameraControllerPlugin);
        app.add_plugin(nightshade_near::NearPlugin {
            settings: nightshade_near::NearSettings {
                cell_size: theme::CELL,
                room: theme::ROOM,
                cell_capacity: theme::CELL_CAPACITY,
            },
        });
        app.add_plugin(nightshade_readout::ReadoutPlugin::titled(theme::TITLE));
        app.add_system(Stage::Startup, systems::setup::stand_it_up);
        app.add_system(Stage::Update, systems::play::walk_the_seeker);
        app.add_system(Stage::Update, systems::play::ask_about_the_seeker);
        app.add_system(Stage::Update, systems::play::take_the_pushes);
        app.add_system(Stage::FrameUpdate, systems::play::report_the_neighbours);
    }
}
