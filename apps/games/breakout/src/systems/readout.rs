use crate::queries;
use crate::resources::RoundState;
use nightshade::prelude::*;
use nightshade_platform::prelude::*;

pub fn show(round: Res<RoundState>, mut window: ResMut<Window>) {
    window.title = queries::readout_text(&round);
}
