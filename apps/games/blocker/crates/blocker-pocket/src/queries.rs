use crate::resources::Inventory;
use blocker_blocks::data::items::{Item, ItemStack, Recipe};
use nightshade_inventory::commands::make_room;
use nightshade_inventory::queries::{held, in_hand};
use nightshade_inventory::{Slot, Slots};

pub fn stack_of(slot: Slot) -> Option<ItemStack> {
    if slot.count == 0 {
        return None;
    }
    Item::from_kind(slot.kind).map(|item| ItemStack {
        item,
        count: slot.count,
    })
}

pub fn slot_of(stack: Option<ItemStack>) -> Slot {
    match stack {
        Some(stack) => Slot {
            kind: stack.item.kind(),
            count: stack.count,
        },
        None => Slot::default(),
    }
}

pub fn stack_at(slots: &Slots, index: usize) -> Option<ItemStack> {
    slots.0.get(index).copied().and_then(stack_of)
}

pub fn stacks_of(slots: &Slots) -> Vec<Option<ItemStack>> {
    slots.0.iter().copied().map(stack_of).collect()
}

pub fn slots_from(stacks: &[Option<ItemStack>], room: usize) -> Slots {
    let mut slots = Slots::default();
    make_room(&mut slots, room);
    for (slot, stack) in slots.0.iter_mut().zip(stacks) {
        *slot = slot_of(*stack);
    }
    slots
}

pub fn selected_stack(inventory: &Inventory) -> Option<ItemStack> {
    in_hand(&inventory.slots, &inventory.selected).and_then(stack_of)
}

pub fn inventory_count(inventory: &Inventory, item: Item) -> u32 {
    held(&inventory.slots, item.kind())
}

pub fn can_craft(inventory: &Inventory, recipe: &Recipe) -> bool {
    recipe
        .inputs
        .iter()
        .all(|(item, count)| inventory_count(inventory, *item) >= *count)
}
