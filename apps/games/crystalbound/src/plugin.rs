use crate::data::Ask;
use crate::resources::{
    Arguments, Battle, Field, Hud, Party, Prompt, Rig, Screen, ScreenRequest, ShownScreen, Toast,
};
use crate::systems;
use crate::theme;
use nightshade::prelude::*;
use nightshade_input::InputPlugin;
use nightshade_platform::prelude::*;
use nightshade_state::prelude::*;

pub struct CrystalboundPlugin;

impl Plugin for CrystalboundPlugin {
    fn build(&self, app: &mut App) {
        app.world.res_mut::<Window>().title = theme::TITLE.to_string();
        let opening = {
            let arguments = app.world.res::<Arguments>();
            if arguments.battle {
                Screen::Battle
            } else if arguments.place.is_some() {
                Screen::Field
            } else {
                Screen::Title
            }
        };
        app.insert_resource(Rig::default());
        app.insert_resource(Field::default());
        app.insert_resource(Party::default());
        app.insert_resource(Battle::default());
        app.insert_resource(Prompt::default());
        app.insert_resource(Toast::default());
        app.insert_resource(Hud::default());
        app.insert_resource(ScreenRequest::default());
        app.insert_resource(ShownScreen::default());
        app.add_plugin(StatePlugin::new(opening));
        app.add_plugin(InputPlugin::<Ask>::new());

        app.add_system(
            Stage::Startup,
            (
                systems::setup::initialize,
                systems::setup::prime_the_battle,
                systems::interface::build,
            ),
        );

        app.on_enter(Screen::Battle, systems::battle::stage_the_battle);
        app.on_exit(Screen::Battle, systems::battle::clear_the_stage);

        app.add_system(
            Stage::Update,
            (
                systems::menus::read_the_state,
                systems::world::build_the_place,
                systems::interface::fade_the_toast,
                systems::effects::bob_the_crystal,
            ),
        );

        app.add_system(
            Stage::Update,
            (
                while_in(
                    Screen::Field,
                    (
                        systems::player::walk_the_party,
                        systems::blade::swing_the_blade,
                        systems::blade::land_the_blows,
                        systems::roamers::stalk_the_party,
                        systems::travel::cross_the_threshold,
                        systems::stations::find_the_prompt,
                        systems::stations::act_on_the_prompt,
                        systems::encounter::count_the_steps,
                        systems::camera::follow_the_walker,
                    ),
                ),
                while_in(
                    Screen::Battle,
                    (
                        systems::battle::run_the_battle,
                        systems::battle::frame_the_battle,
                        systems::battle::dress_the_stage,
                    ),
                ),
            ),
        );

        app.add_system(
            Stage::Update,
            (
                systems::menus::watch_the_party,
                systems::menus::drive_the_menus,
                systems::interface::refresh,
                systems::interface::show_the_screen,
                systems::camera::move_the_camera,
                systems::menus::apply_the_request,
            ),
        );
    }
}
