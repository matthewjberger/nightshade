use crate::contract::{HearthsideCommand, HearthsideCommands};
use crate::data;
use crate::resources::{Aim, Grid, Lot};

pub fn grant(commands: &mut HearthsideCommands, amount: u32) {
    commands.pending.push(HearthsideCommand::Grant { amount });
}

pub fn fill_needs(commands: &mut HearthsideCommands) {
    commands.pending.push(HearthsideCommand::FillNeeds);
}

pub fn select_category(lot: &mut Lot, category: data::Category) {
    if lot.category == category {
        return;
    }
    lot.category = category;
    lot.selected = crate::queries::slots(lot).first().copied().unwrap_or(0);
}

pub fn select_slot(lot: &mut Lot, slot: usize) {
    if let Some(index) = crate::queries::slots(lot).get(slot) {
        lot.selected = *index;
    }
}

pub fn rotate_piece(lot: &mut Lot) {
    lot.yaw_steps = (lot.yaw_steps + 1) % 4;
}

pub fn spend(lot: &mut Lot, price: u32) -> bool {
    if lot.money < price as f32 {
        return false;
    }
    lot.money -= price as f32;
    true
}

pub fn mark_placed(grid: &mut Grid, aim: Aim, piece: usize) {
    match (data::CATALOG[piece].placement, aim) {
        (data::Placement::Floor, Aim::Cell(index)) => grid.floored[index] = true,
        (data::Placement::Cell, Aim::Cell(index)) => grid.cells[index] = Some(piece),
        (data::Placement::Edge, Aim::North(index)) => grid.north[index] = Some(piece),
        (data::Placement::Edge, Aim::West(index)) => grid.west[index] = Some(piece),
        _ => {}
    }
}
