use crate::resources::{Remaining, Running};

pub fn start(remaining: &mut Remaining, running: &mut Running, seconds: f32) {
    remaining.0 = seconds.max(0.0);
    running.0 = remaining.0 > 0.0;
}

pub fn spend(remaining: &mut Remaining, seconds: f32) {
    remaining.0 = (remaining.0 - seconds).max(0.0);
}

pub fn stop(running: &mut Running) {
    running.0 = false;
}
