use crate::data;
use crate::resources::{
    Arguments, Beach, Chat, Fishing, Hud, Pocket, Prompt, Readouts, Rig, Screen, Sheet, Sky, Store,
    Toast, Town, Village, Weather,
};
use crate::systems;
use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_state::prelude::*;

pub struct RoundwoodPlugin;

impl Plugin for RoundwoodPlugin {
    fn build(&self, app: &mut App) {
        let arguments = app.world.res::<Arguments>();
        let fullscreen = arguments.fullscreen;
        let start = if arguments.play {
            Screen::Playing
        } else {
            Screen::Title
        };
        let window = app.world.res_mut::<Window>();
        window.title = data::TITLE.to_string();
        window.use_fullscreen = fullscreen;
        app.insert_resource(Town::default());
        app.insert_resource(Village::default());
        app.insert_resource(Rig::default());
        app.insert_resource(Sheet::default());
        app.insert_resource(Sky::default());
        app.insert_resource(Weather::default());
        app.insert_resource(Prompt::default());
        app.insert_resource(Chat::default());
        app.insert_resource(Pocket::default());
        app.insert_resource(Fishing::default());
        app.insert_resource(Beach::default());
        app.insert_resource(Toast::default());
        app.insert_resource(Store::default());
        app.insert_resource(Hud::default());
        app.insert_resource(Readouts::default());
        app.add_plugin(StatePlugin::new(start));
        app.add_plugin(nightshade_input::InputPlugin::<data::Ask>::new());

        app.add_system(
            Stage::Startup,
            (systems::setup::initialize, systems::interface::build),
        );
        app.add_system(
            Stage::Update,
            (
                while_in(
                    Screen::Playing,
                    (
                        systems::player::walk_the_neighbor,
                        systems::roam::wander,
                        systems::interact::find_the_deed,
                        systems::interact::act_on_the_deed,
                        systems::pickups::hop_and_gather,
                        systems::fishing::wait_for_a_bite,
                    ),
                ),
                while_in(Screen::Talking, (systems::talk::hold_the_conversation,)),
                while_in(Screen::Shop, (systems::shop::keep_the_store,)),
                while_in(Screen::Paused, (systems::menus::drive_the_pause_menu,)),
                while_in(Screen::Title, (systems::menus::drive_the_title_menu,)),
            ),
        );
        app.add_system(
            Stage::Update,
            (
                systems::menus::watch_the_keys,
                systems::menus::hold_the_cursor,
                systems::menus::follow_the_screen,
            ),
        );
        app.add_system(
            Stage::Update,
            (
                systems::place::stand_on_the_world,
                systems::camera::follow_the_neighbor,
                systems::weather::plan_the_weather,
                systems::weather::rain_and_shine,
                systems::sky::turn_the_day,
                systems::sky::light_the_sprites,
                systems::sky::stir_the_air,
                systems::effects::shake_the_trees,
                systems::effects::ripen_the_bushes,
                systems::effects::rest_the_ground,
                systems::effects::wash_up_shells,
                systems::effects::fade_the_toast,
                systems::interface::refresh,
                systems::interface::show_the_pocket,
            ),
        );
    }
}
