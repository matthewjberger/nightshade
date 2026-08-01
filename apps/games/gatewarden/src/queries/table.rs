use crate::theme::{
    CARD_DEPTH_BASE, CARD_DEPTH_STRIDE, CARD_GAP, CARD_WIDTH, HAND_ARC, HAND_FAN, HAND_WIDTH,
    HAND_Y, MINION_GAP, MINION_WIDTH,
};
use nightshade::prelude::*;

pub fn curve_point(source: Vec2, control: Vec2, target: Vec2, along: f32) -> Vec2 {
    let back = 1.0 - along;
    source * back * back + control * 2.0 * back * along + target * along * along
}

pub fn card_depth(rank: f32) -> f32 {
    CARD_DEPTH_BASE + rank * CARD_DEPTH_STRIDE
}

pub fn hand_lean(index: usize, count: usize) -> f32 {
    let middle = count.saturating_sub(1) as f32 * 0.5;
    if middle <= 0.0 {
        return 0.0;
    }
    HAND_FAN * (index as f32 - middle) / middle
}

pub fn hand_slot(index: usize, count: usize) -> Vec2 {
    let span = count as f32 * CARD_WIDTH + count.saturating_sub(1) as f32 * CARD_GAP;
    let start = (HAND_WIDTH - span) * 0.5;
    let middle = count.saturating_sub(1) as f32 * 0.5;
    let offset = if middle > 0.0 {
        (index as f32 - middle) / middle
    } else {
        0.0
    };
    vec2(
        start + index as f32 * (CARD_WIDTH + CARD_GAP),
        HAND_Y - HAND_ARC * (1.0 - offset * offset),
    )
}

pub fn board_slot(index: usize, count: usize, row_y: f32) -> Vec2 {
    let span = count as f32 * MINION_WIDTH + count.saturating_sub(1) as f32 * MINION_GAP;
    let start = (HAND_WIDTH - span) * 0.5;
    vec2(start + index as f32 * (MINION_WIDTH + MINION_GAP), row_y)
}
