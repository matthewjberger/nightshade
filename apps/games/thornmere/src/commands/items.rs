use crate::data::items::{ITEMS, STACK_LIMIT};
use crate::resources::{Bag, Gear, Stack};

pub fn stow(bag: &mut Bag, item: usize, count: u32) -> u32 {
    let mut left = count;
    for slot in bag.slots.iter_mut() {
        if left == 0 {
            break;
        }
        let Some(stack) = slot else {
            continue;
        };
        if stack.item != item || stack.count >= STACK_LIMIT {
            continue;
        }
        let room = STACK_LIMIT - stack.count;
        let moved = room.min(left);
        stack.count += moved;
        left -= moved;
    }
    for slot in bag.slots.iter_mut() {
        if left == 0 {
            break;
        }
        if slot.is_some() {
            continue;
        }
        let moved = STACK_LIMIT.min(left);
        *slot = Some(Stack { item, count: moved });
        left -= moved;
    }
    count - left
}

pub fn take_one(bag: &mut Bag, slot: usize) -> Option<usize> {
    let stack = bag.slots.get_mut(slot)?.as_mut()?;
    let item = stack.item;
    stack.count -= 1;
    if stack.count == 0 {
        bag.slots[slot] = None;
    }
    Some(item)
}

pub fn take_stack(bag: &mut Bag, slot: usize) -> Option<Stack> {
    let stack = (*bag.slots.get(slot)?)?;
    bag.slots[slot] = None;
    Some(stack)
}

pub fn wear_item(bag: &mut Bag, gear: &mut Gear, slot: usize) -> Option<(usize, Option<usize>)> {
    let stack = (*bag.slots.get(slot)?)?;
    let wear = ITEMS[stack.item].wear;
    let place = wear.slot()?;
    let removed = gear.worn[place];
    take_one(bag, slot)?;
    gear.worn[place] = Some(stack.item);
    gear.change += 1;
    if let Some(old) = removed {
        stow(bag, old, 1);
    }
    Some((stack.item, removed))
}

pub fn strip(bag: &mut Bag, gear: &mut Gear, place: usize) -> Option<usize> {
    let item = gear.worn.get(place).copied().flatten()?;
    if bag.slots.iter().all(|slot| slot.is_some()) {
        return None;
    }
    gear.worn[place] = None;
    gear.change += 1;
    stow(bag, item, 1);
    Some(item)
}
