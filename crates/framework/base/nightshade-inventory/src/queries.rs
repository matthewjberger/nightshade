use crate::components::{Selected, Slots};
use crate::data::Slot;

pub fn held(slots: &Slots, kind: u32) -> u32 {
    slots
        .0
        .iter()
        .filter(|slot| slot.kind == kind)
        .map(|slot| slot.count)
        .sum()
}

pub fn in_hand(slots: &Slots, selected: &Selected) -> Option<Slot> {
    slots.0.get(selected.0).copied()
}

pub fn room_for(slots: &Slots, kind: u32, limit: u32) -> u32 {
    slots
        .0
        .iter()
        .map(|slot| {
            if slot.count == 0 {
                limit
            } else if slot.kind == kind {
                limit.saturating_sub(slot.count)
            } else {
                0
            }
        })
        .sum()
}
