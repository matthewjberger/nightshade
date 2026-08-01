use crate::data::items::STACK_LIMIT;
use crate::resources::Bag;

pub fn bag_room(bag: &Bag, item: usize) -> bool {
    bag.slots.iter().any(|slot| match slot {
        None => true,
        Some(stack) => stack.item == item && stack.count < STACK_LIMIT,
    })
}

pub fn bag_used(bag: &Bag) -> usize {
    bag.slots.iter().filter(|slot| slot.is_some()).count()
}
