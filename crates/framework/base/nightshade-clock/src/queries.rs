use crate::resources::Clock;

pub const HOURS_A_DAY: f32 = 24.0;

pub fn hour(clock: &Clock) -> f32 {
    clock.0.rem_euclid(HOURS_A_DAY)
}

pub fn between(clock: &Clock, from: f32, to: f32) -> bool {
    let now = hour(clock);
    if from <= to {
        return now >= from && now < to;
    }
    now >= from || now < to
}

pub fn share_of_day(clock: &Clock) -> f32 {
    hour(clock) / HOURS_A_DAY
}
