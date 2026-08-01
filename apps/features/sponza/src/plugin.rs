use crate::resources::{Handles, Panel, SponzaArguments, Walk};
use crate::systems;
use crate::theme;
use nightshade::prelude::*;
use nightshade_camera_controller::systems::controllers;
use nightshade_platform::prelude::*;

pub struct SponzaPlugin;

impl Plugin for SponzaPlugin {
    fn build(&self, app: &mut App) {
        app.world.res_mut::<Window>().title = theme::TITLE.to_string();
        let arguments = app.world.res::<SponzaArguments>();
        let walk = arguments.walk;
        let panel = arguments.panel;
        app.insert_resource(Walk {
            wanted: walk.then_some(true),
            ..Walk::default()
        });
        app.insert_resource(Handles::default());
        app.add_plugin(nightshade_input::InputPlugin::<crate::data::Ask>::new());
        app.insert_resource(Panel {
            wanted: panel,
            rate_in_title_on: true,
            ..Panel::default()
        });
        app.add_system(Stage::Startup, systems::setup::initialize);
        app.add_system(
            Stage::Update,
            (
                systems::setup::lay_the_floor,
                systems::panel::toggle_the_panel,
                systems::panel::read_the_panel,
                systems::panel::refresh_the_rate,
                systems::walk::toggle_the_walk,
                systems::focus::follow_the_look,
            ),
        );
        app.add_system(
            Stage::FrameUpdate,
            (
                controllers::apply_camera_requests_system,
                controllers::first_person_camera_look_system,
            ),
        );
    }
}
