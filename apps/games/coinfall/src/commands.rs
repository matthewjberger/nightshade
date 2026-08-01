use crate::data;
use crate::resources::{RunState, Screen};
use crate::theme;

pub fn reset_run(run: &mut RunState) {
    run.seconds = data::LEVELS[run.level].seconds;
    run.coins = 0;
    run.score = 0;
    run.restart_wanted = false;
}

pub fn spend_seconds(run: &mut RunState, seconds: f32) {
    run.seconds = (run.seconds - seconds).max(0.0);
}

pub fn take_coin(run: &mut RunState) {
    run.coins += 1;
    run.seconds += theme::COIN_SECONDS;
}

pub fn take_stomp(run: &mut RunState) {
    run.coins += theme::STOMP_COINS;
    run.seconds += theme::COIN_SECONDS * theme::STOMP_COINS as f32;
}

pub fn clear_run(run: &mut RunState) {
    run.score = run.seconds.ceil() as u32;
    let best = &mut run.bests[run.level];
    *best = (*best).max(run.score);
    if run.level + 1 < data::LEVELS.len() {
        run.unlocked = run.unlocked.max(run.level + 2);
    }
}

pub fn want_restart(run: &mut RunState) {
    run.restart_wanted = true;
}

pub fn want_screen(run: &mut RunState, screen: Screen) {
    run.wanted_screen = Some(screen);
}

pub fn want_level(run: &mut RunState, level: usize) {
    run.level = level.min(data::LEVELS.len() - 1);
    run.wanted_level = Some(run.level);
}

pub fn move_selection(run: &mut RunState, step: i32) {
    let last = run.unlocked.min(data::LEVELS.len()) - 1;
    let wanted = run.selected as i32 + step;
    run.selected = wanted.clamp(0, last as i32) as usize;
}
