use crate::resources::RunState;

pub fn record_best(run: &mut RunState, speed: f32) {
    if speed > run.best_speed {
        run.best_speed = speed;
    }
}

pub fn record_release(run: &mut RunState, speed: f32) {
    run.last_release = speed;
}

pub fn count_fall(run: &mut RunState) {
    run.falls = run.falls.saturating_add(1);
}

pub fn clear_run(run: &mut RunState) {
    run.best_speed = 0.0;
    run.last_release = 0.0;
    run.falls = 0;
}
