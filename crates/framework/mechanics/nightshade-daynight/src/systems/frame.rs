use crate::resources::DaySettings;
use nightshade::prelude::*;
use nightshade_clock::Clock;
use nightshade_platform::contract::time::Time;

pub fn turn_the_hour(time: Res<Time>, settings: Res<DaySettings>, mut clock: ResMut<Clock>) {
    clock.0 = (clock.0 + settings.hours_a_second * time.delta_time).rem_euclid(24.0);
}
