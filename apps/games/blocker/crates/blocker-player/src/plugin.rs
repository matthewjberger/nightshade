use crate::data::ask::Ask;
use crate::data::settings::Settings;
use crate::resources::{Aim, Cursor, Fireworks, Hand, Markers, Mining, Profile, Prompt, Resume};
use crate::systems;
use blocker_screen::data::screen::Screen;
use nightshade::prelude::*;
use nightshade_config::prelude::ConfigPlugin;
use nightshade_input::InputPlugin;
use nightshade_state::prelude::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn needs(&self) -> Vec<nightshade::plugin::Need> {
        nightshade::needs![nightshade_random::Random => "the app plugin"]
    }

    fn build(&self, app: &mut App) {
        app.add_plugin(InputPlugin::<Ask>::new());
        app.add_plugin(blocker_figure::plugin::FigurePlugin);
        app.add_plugin(ConfigPlugin::<Settings>::new().persist("blocker", "player.toml"));
        app.insert_resource(Cursor::default());
        app.insert_resource(Aim::default());
        app.insert_resource(Mining::default());
        app.insert_resource(Markers::default());
        app.insert_resource(Hand::default());
        app.insert_resource(Prompt::default());
        app.insert_resource(Profile::default());
        app.insert_resource(Fireworks::default());
        app.insert_resource(Resume::default());
        app.add_system(Stage::Startup, systems::arrive::place_player);
        app.add_system(
            Stage::Update,
            while_in(
                Screen::Playing,
                (
                    systems::boats::sail_the_boats,
                    systems::player::control,
                    systems::interact::slots::pick_the_slot,
                    systems::interact::aim,
                    systems::interact::paint_the_crack,
                    systems::interact::tools::use_item,
                    systems::interact::prompt::find_the_prompt,
                    systems::interact::prompt::open_the_chest,
                    systems::interact::prompt::act_on_the_prompt,
                    systems::sight::toggle_the_nameplates,
                    systems::vitals::keep_the_body,
                ),
            ),
        );
        app.add_system(
            Stage::Update,
            (
                systems::fresh::stand_anew,
                systems::arrive::stand_the_player,
                systems::cursor::keep_the_cursor,
                systems::sight::aim_the_viewer,
                systems::gravity::drop_the_fallen,
                systems::fireworks::fly_the_shells,
                systems::items::drift,
                systems::hand::hold,
                systems::figure::stride_the_body,
                systems::figure::carry_the_glider,
                systems::figure::name_the_player,
                systems::figure::show_the_body,
            ),
        );
    }
}
