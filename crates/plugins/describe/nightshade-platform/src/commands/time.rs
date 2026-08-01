use crate::contract::time::Time;

pub fn set_time_scale(time: &mut Time, scale: f32) {
    time.time_speed = scale.max(0.0);
}

pub fn pause(time: &mut Time) {
    time.paused = true;
}

pub fn unpause(time: &mut Time) {
    time.paused = false;
}

pub fn set_paused(time: &mut Time, paused: bool) {
    time.paused = paused;
}
