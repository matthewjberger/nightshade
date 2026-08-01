use crate::queries::{can_craft, inventory_count, slot_of, stack_at, stack_of};
use crate::resources::{Chests, Inventory};
use blocker_blocks::data::items::{Item, ItemStack, Recipe};
use nightshade_inventory::commands::{add, make_room, select, swap, take};
use nightshade_inventory::{Slot, Slots, StackLimit};

pub fn stack_into(slots: &mut Slots, item: Item, count: u32) -> u32 {
    let stored = add(slots, &StackLimit(item.max_stack()), item.kind(), count);
    count - stored
}

pub fn inventory_add(inventory: &mut Inventory, item: Item, count: u32) -> u32 {
    stack_into(&mut inventory.slots, item, count)
}

pub fn inventory_take(inventory: &mut Inventory, item: Item, count: u32) -> bool {
    if inventory_count(inventory, item) < count {
        return false;
    }
    take(&mut inventory.slots, item.kind(), count);
    true
}

pub fn clear_slot(slots: &mut Slots, index: usize) {
    if let Some(slot) = slots.0.get_mut(index) {
        *slot = Slot::default();
    }
}

pub fn put_stack(slots: &mut Slots, index: usize, stack: Option<ItemStack>) {
    if let Some(slot) = slots.0.get_mut(index) {
        *slot = slot_of(stack);
    }
}

pub fn inventory_take_selected(inventory: &mut Inventory, count: u32) -> Option<ItemStack> {
    let index = inventory.selected.0;
    let stack = stack_at(&inventory.slots, index)?;
    let taken = stack.count.min(count);
    let slot = inventory.slots.0.get_mut(index)?;
    slot.count -= taken;
    if slot.count == 0 {
        *slot = Slot::default();
    }
    Some(ItemStack {
        item: stack.item,
        count: taken,
    })
}

pub fn inventory_swap_selected(inventory: &mut Inventory, item: Item) {
    let index = inventory.selected.0;
    let count = stack_at(&inventory.slots, index).map_or(0, |stack| stack.count);
    if count > 1 {
        if let Some(slot) = inventory.slots.0.get_mut(index) {
            slot.count -= 1;
        }
        inventory_add(inventory, item, 1);
        return;
    }
    put_stack(
        &mut inventory.slots,
        index,
        Some(ItemStack { item, count: 1 }),
    );
}

pub fn swap_slots(inventory: &mut Inventory, first: usize, second: usize) {
    swap(&mut inventory.slots, first, second);
}

pub fn select_slot(inventory: &mut Inventory, index: usize) {
    select(
        &mut inventory.selected,
        &inventory.slots,
        index.min(blocker_blocks::data::items::HOTBAR_SLOTS - 1),
    );
}

pub fn select_next_slot(inventory: &mut Inventory, steps: i32) {
    let slots = blocker_blocks::data::items::HOTBAR_SLOTS as i32;
    inventory.selected.0 = (inventory.selected.0 as i32 + steps).rem_euclid(slots) as usize;
}

pub fn craft(inventory: &mut Inventory, recipe: &Recipe) -> bool {
    if !can_craft(inventory, recipe) {
        return false;
    }
    for (item, count) in recipe.inputs {
        inventory_take(inventory, *item, *count);
    }
    let leftover = inventory_add(inventory, recipe.output, recipe.count);
    if leftover > 0 {
        for (item, count) in recipe.inputs {
            inventory_add(inventory, *item, *count);
        }
        inventory_take(inventory, recipe.output, recipe.count - leftover);
        return false;
    }
    true
}

pub fn chest_slots(chests: &mut Chests, key: [i32; 3], room: usize) -> &mut Slots {
    chests.0.entry(key).or_insert_with(|| {
        let mut slots = Slots::default();
        make_room(&mut slots, room);
        slots
    })
}

pub fn stock_the_slots(slots: &mut Slots, goods: &[(Item, u32)]) {
    for (slot, (item, count)) in slots.0.iter_mut().zip(goods) {
        if stack_of(*slot).is_none() {
            *slot = Slot {
                kind: item.kind(),
                count: *count,
            };
        }
    }
}
