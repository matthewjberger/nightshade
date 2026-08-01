use crate::contract::CrosstownCommands;
use crate::data;
use crate::resources::{Arguments, Interface, Models, Ring, Screen, Sheets, Talk};
use crate::systems;
use nightshade::prelude::*;
use nightshade_input::InputPlugin;
use nightshade_platform::prelude::*;
use nightshade_state::prelude::*;

pub struct CrosstownPlugin;

impl Plugin for CrosstownPlugin {
    fn build(&self, app: &mut App) {
        app.world.res_mut::<Window>().title = data::TITLE.to_string();
        app.insert_resource(CrosstownCommands::default());
        app.insert_resource(Sheets::default());
        app.insert_resource(Models::default());
        app.insert_resource(Ring::default());
        app.insert_resource(Interface::default());
        app.insert_resource(Talk::default());
        let opening = if app.world.res::<Arguments>().play {
            Screen::Play
        } else {
            Screen::Title
        };
        app.add_plugin(StatePlugin::new(opening));
        app.add_plugin(InputPlugin::<data::Ask>::new());

        app.add_system(Stage::Startup, systems::setup::build_interface);
        app.add_system(Stage::Startup, systems::setup::dress_scene);
        app.add_system(Stage::Startup, systems::setup::request_models);
        app.add_system(Stage::Startup, systems::setup::build_world);

        app.on_enter(Screen::Title, systems::screens::open_title);
        app.on_enter(Screen::Play, systems::screens::open_play);
        app.on_enter(Screen::Pause, systems::screens::open_pause);
        app.on_enter(Screen::Over, systems::screens::open_over);

        app.add_system(
            Stage::Update,
            while_in(Screen::Title, systems::screens::choose_title),
        );
        app.add_system(
            Stage::Update,
            while_in(Screen::Pause, systems::screens::choose_pause),
        );
        app.add_system(
            Stage::Update,
            while_in(Screen::Over, systems::screens::choose_over),
        );
        app.add_system(
            Stage::Update,
            while_in(
                Screen::Play,
                (
                    systems::screens::reset,
                    systems::stages::raise,
                    systems::stages::open,
                    systems::screens::hold,
                    systems::player::ask,
                    systems::player::fight,
                    systems::player::hop,
                    systems::flight::fall,
                    systems::flight::tumble,
                    systems::player::stroll,
                ),
            ),
        );
        app.add_system(
            Stage::Update,
            while_in(
                Screen::Play,
                (
                    systems::thugs::approach,
                    systems::thugs::swing,
                    systems::combat::wind,
                    systems::combat::land,
                    systems::combat::sweep,
                    systems::combat::splinter,
                    systems::combat::hurt,
                    systems::crowd::part,
                    systems::crowd::clear,
                    systems::pickups::gather,
                    systems::pickups::fly,
                    systems::ring::keep,
                    systems::ring::watch,
                    systems::stages::advance,
                    systems::camera::follow,
                ),
            ),
        );
        app.add_system(
            Stage::Update,
            (
                systems::props::place,
                systems::pose::dress,
                systems::head_up::show,
                systems::banter::speak,
            ),
        );
    }
}
