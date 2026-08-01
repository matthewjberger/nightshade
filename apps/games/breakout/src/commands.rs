use crate::data;
use crate::resources::RoundState;

pub fn next_random(round: &mut RoundState) -> f32 {
    round.seed = round
        .seed
        .wrapping_mul(1_664_525)
        .wrapping_add(1_013_904_223);
    (round.seed >> 16) as f32 / 65_535.0 * 2.0 - 1.0
}

pub fn score_points(round: &mut RoundState, points: u32) {
    round.score = round.score.saturating_add(points);
    round.bricks_standing = round.bricks_standing.saturating_sub(1);
    if round.bricks_standing == 0 {
        round.serving = true;
        round.rebuild_wanted = true;
    }
}

pub fn lose_life(round: &mut RoundState) {
    round.lives = round.lives.saturating_sub(1);
    round.serving = true;
    if round.lives == 0 {
        restart(round);
    }
}

pub fn restart(round: &mut RoundState) {
    round.score = 0;
    round.lives = data::STARTING_LIVES;
    round.serving = true;
    round.rebuild_wanted = true;
}
