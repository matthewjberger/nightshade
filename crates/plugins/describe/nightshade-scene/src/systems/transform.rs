use crate::contract::GlobalTransform;
use crate::queries::computed_global_transform;

use nightshade_ecs::system_param::*;

pub fn update_global_transforms_system(
    mut state: ResMut<crate::resources::transform::TransformState>,
    mut hierarchy: ResMut<nightshade_ecs::HierarchyIndex>,
    storage: &mut nightshade_ecs::Storage,
) {
    hierarchy.sync(storage);

    let initial_pass = !state.initial_pass_done;

    let mut dirty_entities = std::mem::take(&mut state.dirty_scratch);
    dirty_entities.clear();

    if initial_pass {
        dirty_entities.extend(storage.query_entities::<(
            crate::contract::LocalTransform,
            crate::contract::GlobalTransform,
        )>());
        state.initial_pass_done = true;
    } else {
        let since_tick = state.last_consumed_tick;
        dirty_entities.extend(
            storage.query_entities_changed_since::<(crate::contract::LocalTransform,)>(since_tick),
        );
        dirty_entities
            .extend(storage.query_entities_changed_since::<(nightshade_ecs::ChildOf,)>(since_tick));
    }
    state.last_consumed_tick = storage.current_tick();
    storage.increment_tick();

    if dirty_entities.is_empty() {
        state.dirty_scratch = dirty_entities;
        return;
    }

    dirty_entities.sort_unstable_by_key(|entity| (entity.id, entity.generation));
    dirty_entities.dedup();

    let mut seen = std::mem::take(&mut state.expansion_seen);
    seen.clear();
    seen.extend(dirty_entities.iter().copied());
    let mut index = 0;
    while index < dirty_entities.len() {
        let entity = dirty_entities[index];
        index += 1;
        for child in hierarchy.children(entity) {
            if !frozen(storage, *child) && seen.insert(*child) {
                dirty_entities.push(*child);
            }
        }
    }

    let mut stack = std::mem::take(&mut state.traversal_stack);
    let mut moved = std::mem::take(&mut state.moved_entities);
    stack.clear();
    moved.clear();

    for entity in dirty_entities.iter().copied() {
        if frozen(storage, entity) {
            seen.remove(&entity);
            continue;
        }
        let parent_is_dirty = storage
            .get::<nightshade_ecs::ChildOf>(entity)
            .map(|child_of| child_of.0)
            .is_some_and(|parent_entity| seen.contains(&parent_entity));
        if !parent_is_dirty {
            stack.push(entity);
        }
    }

    while let Some(current) = stack.pop() {
        if !seen.remove(&current) {
            continue;
        }

        let new_global_transform = computed_global_transform(storage, current);
        if let Some(global_transform) = storage.get_mut::<crate::contract::GlobalTransform>(current)
        {
            *global_transform = GlobalTransform(new_global_transform);
        }

        moved.push(current);

        for child in hierarchy.children(current) {
            if !frozen(storage, *child) {
                stack.push(*child);
            }
        }
    }

    seen.clear();
    stack.clear();
    dirty_entities.clear();
    state.expansion_seen = seen;
    state.traversal_stack = stack;
    state.moved_entities = moved;
    state.dirty_scratch = dirty_entities;
}

fn frozen(storage: &nightshade_ecs::Storage, entity: nightshade_ecs::Entity) -> bool {
    storage
        .get::<crate::contract::Frozen>(entity)
        .is_some_and(|frozen| frozen.0)
}
