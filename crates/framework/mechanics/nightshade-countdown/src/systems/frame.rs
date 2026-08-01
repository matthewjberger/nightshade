use crate::data::RanOut;
use crate::resources::{Remaining, Running};
use nightshade::ecs::system_param::EventWriter;
use nightshade::prelude::*;
use nightshade_platform::contract::time::Time;

pub fn spend_the_time(
    time: Res<Time>,
    mut remaining: ResMut<Remaining>,
    mut running: ResMut<Running>,
    mut ran_out: EventWriter<RanOut>,
) {
    if !running.0 {
        return;
    }
    remaining.0 = (remaining.0 - time.delta_time).max(0.0);
    if remaining.0 <= 0.0 {
        running.0 = false;
        ran_out.send(RanOut);
    }
}
