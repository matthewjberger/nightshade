use crate::data;
use crate::queries;
use crate::resources::{Juice, Outcome, RoundState};

pub fn next_random(round: &mut RoundState) -> f32 {
    round.seed = round
        .seed
        .wrapping_mul(1_664_525)
        .wrapping_add(1_013_904_223);
    (round.seed >> 16) as f32 / 65_535.0 * 2.0 - 1.0
}

pub fn chip_brick(round: &mut RoundState) {
    round.score = round.score.saturating_add(1);
}

pub fn score_points(round: &mut RoundState, points: u32) {
    round.score = round.score.saturating_add(points);
    round.bricks_standing = round.bricks_standing.saturating_sub(1);
}

pub fn lose_life(round: &mut RoundState) {
    round.lives = round.lives.saturating_sub(1);
    round.serving = true;
    if round.lives == 0 {
        round.outcome = Some(Outcome::Lost);
    }
}

pub fn clear_level(round: &mut RoundState) -> u32 {
    let bonus = if queries::beat_par(round) {
        data::PAR_BONUS
    } else {
        0
    };
    round.score = round.score.saturating_add(bonus);
    round.level_time = 0.0;
    round.serving = true;
    round.rebuild_wanted = true;
    if round.level + 1 >= data::LEVEL_COUNT {
        round.outcome = Some(Outcome::Won);
        round.reached = data::LEVEL_COUNT - 1;
        round.unlocked = data::LEVEL_COUNT - 1;
        round.level = 0;
    } else {
        round.level += 1;
        round.reached = round.level;
        round.unlocked = round.unlocked.max(round.level);
    }
    bonus
}

pub fn choose_next_level(round: &mut RoundState) {
    round.chosen = if round.chosen >= round.unlocked {
        0
    } else {
        round.chosen + 1
    };
}

pub fn restart(round: &mut RoundState) {
    round.score = 0;
    round.lives = data::STARTING_LIVES;
    round.level = round.chosen;
    round.reached = round.chosen;
    round.level_time = 0.0;
    round.serving = true;
    round.rebuild_wanted = true;
    round.outcome = None;
}

pub fn shake(juice: &mut Juice, amount: f32) {
    juice.shake = (juice.shake + amount).min(data::SHAKE_LIMIT);
}

pub fn tint_toward(juice: &mut Juice, color: [f32; 4]) {
    juice.tint_target = queries::tint_of(color);
}

pub fn hold_slow(juice: &mut Juice) {
    juice.slow_hold = data::SLOW_SECONDS;
}
