use crate::data::settings::Settings;
use crate::resources::Quarry;
use crate::systems;
use blocker_screen::data::screen::Screen;
use nightshade::prelude::*;
use nightshade_config::prelude::ConfigPlugin;
use nightshade_state::prelude::*;

pub struct BeastsPlugin;

impl Plugin for BeastsPlugin {
    fn needs(&self) -> Vec<nightshade::plugin::Need> {
        nightshade::needs![nightshade_random::Random => "the app plugin"]
    }

    fn build(&self, app: &mut App) {
        app.add_plugin(ConfigPlugin::<Settings>::new().persist("blocker", "beasts.toml"));
        app.insert_resource(Quarry::default());
        app.add_system(Stage::Startup, systems::beast_art::paint_the_beasts);
        app.add_system(Stage::Update, systems::fresh::clear_the_beasts);
        app.add_system(
            Stage::Update,
            while_in(
                Screen::Playing,
                (
                    systems::asks::answer_the_asks,
                    systems::beasts::roam::target_nearest,
                    systems::beasts::stir_the_beasts,
                    systems::beasts::roam::roam_the_beasts,
                    systems::beasts::fight::fight_the_beasts,
                    systems::beasts::fight::shove_the_beasts,
                    systems::beasts::reap_the_beasts,
                    systems::beasts::fight::pose_the_beasts,
                    systems::beasts::fight::flinch_the_beasts,
                    systems::beasts::plates::label_the_beasts,
                ),
            ),
        );
    }
}
