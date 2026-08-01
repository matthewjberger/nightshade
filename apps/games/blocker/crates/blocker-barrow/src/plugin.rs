use crate::resources::{Counting, Den};
use crate::systems;
use blocker_screen::data::screen::Screen;
use nightshade::prelude::*;
use nightshade_state::prelude::*;

pub struct BarrowPlugin;

impl Plugin for BarrowPlugin {
    fn needs(&self) -> Vec<nightshade::plugin::Need> {
        nightshade::needs![nightshade_random::Random => "the app plugin"]
    }

    fn build(&self, app: &mut App) {
        app.insert_resource(Den::default());
        app.insert_resource(Counting::default());
        app.add_system(Stage::Update, systems::fresh::clear_the_den);
        app.add_system(
            Stage::Update,
            while_in(
                Screen::Playing,
                (
                    systems::barrow::stock_the_barrows,
                    systems::barrow::fill_the_hoard,
                    systems::barrow::name_the_barrow,
                    systems::barrow::point_to_the_watch,
                    systems::barrow::stand_the_waiting,
                    systems::barrow::hear_the_counting,
                ),
            ),
        );
        app.add_system(
            Stage::Update,
            while_in(
                Screen::Playing,
                (
                    systems::trail::aim_the_waypoint,
                    systems::trail::walk_the_trail,
                    systems::trail::stock_the_camps,
                    systems::trail::free_the_watchman,
                ),
            ),
        );
    }
}
