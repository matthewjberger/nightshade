use crate::queries;
use crate::resources::Readout;
use nightshade::prelude::*;
use nightshade_platform::contract::time::Time;
use nightshade_platform::contract::window::Window;

pub fn show_the_readout(time: Res<Time>, readout: Res<Readout>, mut window: ResMut<Window>) {
    window.title = queries::line(&readout, time.frames_per_second);
}
