use crate::queries::slots_from;
use nightshade_inventory::{Selected, Slots};
use std::collections::HashMap;

pub struct Inventory {
    pub slots: Slots,
    pub selected: Selected,
}

impl Default for Inventory {
    fn default() -> Self {
        Self {
            slots: slots_from(
                &blocker_blocks::data::items::starting_inventory(),
                blocker_blocks::data::items::INVENTORY_SLOTS,
            ),
            selected: Selected(0),
        }
    }
}

#[derive(Default)]
pub struct Chests(pub HashMap<[i32; 3], Slots>);

#[derive(Default, Clone, Copy, Debug)]
pub struct OpenChest(pub Option<[i32; 3]>);
