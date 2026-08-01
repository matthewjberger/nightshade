use crate::contract::Cell;
use crate::queries;
use crate::resources::Bulwark;
use crate::theme;
use nightshade::prelude::Entity;

pub fn take_the_cell(bulwark: &mut Bulwark, cell: Cell, tower: Entity, cost: u32) {
    let index = queries::cell_index(cell);
    if let Some(slot) = bulwark.taken.get_mut(index) {
        *slot = Some(tower);
    }
    bulwark.coin = bulwark.coin.saturating_sub(cost);
}

pub fn free_the_cell(bulwark: &mut Bulwark, post: u32, refund: u32) {
    if let Some(slot) = bulwark.taken.get_mut(post as usize) {
        *slot = None;
    }
    bulwark.coin = bulwark.coin.saturating_add(refund);
}

pub fn pay_for_the_mend(bulwark: &mut Bulwark) {
    let cost = queries::mend_cost(bulwark.mends);
    bulwark.coin = bulwark.coin.saturating_sub(cost);
    bulwark.mends += 1;
    bulwark.lives = (bulwark.lives + 1).min(theme::STARTING_LIVES);
}

pub fn pay_the_cost(bulwark: &mut Bulwark, cost: u32) {
    bulwark.coin = bulwark.coin.saturating_sub(cost);
}

pub fn pay_the_bounty(bulwark: &mut Bulwark, bounty: u32) {
    bulwark.coin = bulwark.coin.saturating_add(bounty);
    bulwark.kills = bulwark.kills.saturating_add(1);
}

pub fn lose_a_life(bulwark: &mut Bulwark) {
    bulwark.lives = bulwark.lives.saturating_sub(1);
    bulwark.leaked = bulwark.leaked.saturating_add(1);
}
