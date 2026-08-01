use crate::data::SculptJob;
use crate::resources::{Gallery, Panel};
use crate::systems;
use crate::theme;
use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_worker::workers_hire;

pub struct WorkersPlugin;

impl Plugin for WorkersPlugin {
    fn build(&self, app: &mut App) {
        app.world.res_mut::<Window>().title = theme::TITLE.to_string();
        let gallery = Gallery::default();
        let hands = gallery.hands;
        app.insert_resource(gallery);
        app.insert_resource(Panel::default());
        app.insert_resource(workers_hire::<SculptJob>(hands));
        app.add_system(Stage::Startup, systems::setup::initialize);
        app.add_system(
            Stage::Update,
            (
                systems::view::turn_the_camera,
                systems::panel::drive_the_panel,
                systems::gallery::ask_the_crew,
                systems::gallery::take_the_statues,
                systems::gallery::toil_on_the_frame,
                systems::markers::show_the_hands,
                systems::panel::refresh_the_panel,
            ),
        );
    }
}
