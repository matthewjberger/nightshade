use nightshade_paint::resources::RenderSlotAllocator;

pub fn allocate_rect_slot(
    allocator: &mut RenderSlotAllocator,
    entity: nightshade::ecs::Entity,
) -> u32 {
    if let Some(&slot) = allocator.rect_slots.get(&entity) {
        return slot;
    }
    let slot = allocator.free_rect_slots.pop().unwrap_or_else(|| {
        let next_slot = allocator.next_rect_slot;
        allocator.next_rect_slot += 1;
        next_slot
    });
    allocator.rect_slots.insert(entity, slot);
    allocator.dirty_rect_slots.insert(slot);
    slot
}

pub fn release_rect_slot(allocator: &mut RenderSlotAllocator, entity: nightshade::ecs::Entity) {
    if let Some(slot) = allocator.rect_slots.remove(&entity) {
        allocator.free_rect_slots.push(slot);
        allocator.dirty_rect_slots.remove(&slot);
    }
}

pub fn allocate_image_slot(
    allocator: &mut RenderSlotAllocator,
    entity: nightshade::ecs::Entity,
) -> u32 {
    if let Some(&slot) = allocator.image_slots.get(&entity) {
        return slot;
    }
    let slot = allocator.free_image_slots.pop().unwrap_or_else(|| {
        let next_slot = allocator.next_image_slot;
        allocator.next_image_slot += 1;
        next_slot
    });
    allocator.image_slots.insert(entity, slot);
    allocator.dirty_image_slots.insert(slot);
    slot
}

pub fn release_image_slot(allocator: &mut RenderSlotAllocator, entity: nightshade::ecs::Entity) {
    if let Some(slot) = allocator.image_slots.remove(&entity) {
        allocator.free_image_slots.push(slot);
        allocator.dirty_image_slots.remove(&slot);
    }
}

pub fn allocate_text_slot(
    allocator: &mut RenderSlotAllocator,
    entity: nightshade::ecs::Entity,
) -> u32 {
    if let Some(&slot) = allocator.text_slots.get(&entity) {
        return slot;
    }
    let slot = allocator.free_text_slots.pop().unwrap_or_else(|| {
        let next_slot = allocator.next_text_slot;
        allocator.next_text_slot += 1;
        next_slot
    });
    allocator.text_slots.insert(entity, slot);
    allocator.dirty_text_slots.insert(slot);
    slot
}

pub fn release_text_slot(allocator: &mut RenderSlotAllocator, entity: nightshade::ecs::Entity) {
    if let Some(slot) = allocator.text_slots.remove(&entity) {
        allocator.free_text_slots.push(slot);
        allocator.dirty_text_slots.remove(&slot);
    }
}
