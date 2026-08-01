use crate::queries::HOURS_A_DAY;
use crate::resources::Clock;

pub fn set_hour(clock: &mut Clock, hour: f32) {
    clock.0 = hour.rem_euclid(HOURS_A_DAY);
}

pub fn pass(clock: &mut Clock, hours: f32) {
    clock.0 = (clock.0 + hours).rem_euclid(HOURS_A_DAY);
}
