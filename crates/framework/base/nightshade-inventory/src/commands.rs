use crate::components::{Selected, Slots};
use crate::data::Slot;
use crate::resources::StackLimit;

pub fn add(slots: &mut Slots, limit: &StackLimit, kind: u32, count: u32) -> u32 {
    let mut left = count;
    for slot in slots.0.iter_mut() {
        if left == 0 {
            break;
        }
        if slot.kind == kind && slot.count < limit.0 {
            let room = limit.0 - slot.count;
            let moved = room.min(left);
            slot.count += moved;
            left -= moved;
        }
    }
    for slot in slots.0.iter_mut() {
        if left == 0 {
            break;
        }
        if slot.count == 0 {
            let moved = limit.0.min(left);
            slot.kind = kind;
            slot.count = moved;
            left -= moved;
        }
    }
    count - left
}

pub fn take(slots: &mut Slots, kind: u32, count: u32) -> u32 {
    let mut left = count;
    for slot in slots.0.iter_mut() {
        if left == 0 {
            break;
        }
        if slot.kind != kind || slot.count == 0 {
            continue;
        }
        let moved = slot.count.min(left);
        slot.count -= moved;
        left -= moved;
        if slot.count == 0 {
            slot.kind = 0;
        }
    }
    count - left
}

pub fn swap(slots: &mut Slots, one: usize, other: usize) {
    if one >= slots.0.len() || other >= slots.0.len() {
        return;
    }
    slots.0.swap(one, other);
}

pub fn select(selected: &mut Selected, slots: &Slots, index: usize) {
    if slots.0.is_empty() {
        return;
    }
    selected.0 = index.min(slots.0.len() - 1);
}

pub fn select_next(selected: &mut Selected, slots: &Slots) {
    if slots.0.is_empty() {
        return;
    }
    selected.0 = (selected.0 + 1) % slots.0.len();
}

pub fn make_room(slots: &mut Slots, count: usize) {
    slots.0 = vec![Slot::default(); count];
}
