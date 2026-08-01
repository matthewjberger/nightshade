use crate::systems;
use blocker_screen::data::screen::Screen;
use nightshade::prelude::*;
use nightshade_state::prelude::*;

pub struct FarmPlugin;

impl Plugin for FarmPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(
            Stage::Update,
            (systems::fresh::clear_the_fields, systems::farming::grow),
        );
        app.add_system(
            Stage::Update,
            while_in(
                Screen::Playing,
                (systems::sowing::sow, systems::sowing::offer_the_harvest),
            ),
        );
    }
}
