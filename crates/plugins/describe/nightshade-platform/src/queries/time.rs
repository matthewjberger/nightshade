use crate::contract::time::Time;

pub fn time_scale(time: &Time) -> f32 {
    time.time_speed
}

pub fn is_paused(time: &Time) -> bool {
    time.paused
}
