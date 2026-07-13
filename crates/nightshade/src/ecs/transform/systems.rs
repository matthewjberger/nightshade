pub use crate::ecs::transform::queries::*;

use crate::ecs::world::{
    GLOBAL_TRANSFORM, INSTANCED_MESH, LOCAL_TRANSFORM, PARENT, RENDER_MESH, World,
    components::{GlobalTransform, Parent},
};

use crate::prelude::*;
pub fn run_systems(world: &mut World) {
    let _span = tracing::info_span!("transform_systems").entered();
    update_global_transforms_system(world);
}

pub fn update_parent(world: &mut World, child: freecs::Entity, new_parent: Option<Parent>) {
    let old_parent_entity = world.ecs.worlds[CORE]
        .get::<crate::ecs::transform::components::Parent>(child)
        .and_then(|p| p.0);

    if let Some(old_parent_entity) = old_parent_entity
        && let Some(children) = world
            .resources
            .transform_state
            .children_cache
            .get_mut(&old_parent_entity)
    {
        children.retain(|&e| e != child);
    }

    if let Some(new_parent) = new_parent {
        world.ecs.worlds[CORE].set(child, new_parent);
        if let Some(parent_entity) = new_parent.0 {
            world
                .resources
                .transform_state
                .children_cache
                .entry(parent_entity)
                .or_default()
                .push(child);
        }
    } else {
        world.ecs.worlds[CORE].remove_components(child, PARENT);
    }

    world.resources.transform_state.children_cache_valid = false;
}

pub fn validate_and_rebuild_children_cache(world: &mut World) {
    world.resources.transform_state.children_cache.clear();

    let children_cache = &mut world.resources.transform_state.children_cache;
    for (child, (parent,)) in world.ecs.worlds[CORE].query_ref::<&Parent>().iter() {
        if let Some(parent_entity) = parent.0 {
            children_cache.entry(parent_entity).or_default().push(child);
        }
    }

    for children in world.resources.transform_state.children_cache.values_mut() {
        children.sort_by_key(|entity| entity.id);
    }

    world.resources.transform_state.children_cache_valid = true;
}

/// Recomputes [`GlobalTransform`] for every entity whose [`LocalTransform`] or
/// [`Parent`] changed since the previous run, plus all of their descendants.
/// Change detection rides on the ECS change ticks, so any write through the
/// component accessors is picked up automatically. The system keeps its own
/// change cursor and fences the world tick after consuming, so writes made
/// later in the frame land in the next run regardless of how often other
/// consumers advance the tick. Propagation walks each dirty subtree top down,
/// multiplying the parent's already-updated global by the child's local, one
/// matrix product per entity.
pub fn update_global_transforms_system(world: &mut World) {
    let initial_pass = !world.resources.transform_state.initial_pass_done;

    let mut dirty_entities = std::mem::take(&mut world.resources.transform_state.dirty_scratch);
    dirty_entities.clear();

    let mut hierarchy_changed = false;
    if initial_pass {
        dirty_entities
            .extend(world.ecs.worlds[CORE].query_entities(LOCAL_TRANSFORM | GLOBAL_TRANSFORM));
        world.resources.transform_state.initial_pass_done = true;
    } else {
        let since_tick = world.resources.transform_state.last_consumed_tick;
        dirty_entities.extend(
            world.ecs.worlds[CORE].query_entities_changed_since(LOCAL_TRANSFORM, since_tick),
        );
        let local_changed_count = dirty_entities.len();
        dirty_entities
            .extend(world.ecs.worlds[CORE].query_entities_changed_since(PARENT, since_tick));
        hierarchy_changed = dirty_entities.len() > local_changed_count;
    }
    world.resources.transform_state.last_consumed_tick = world.ecs.worlds[CORE].current_tick();
    world.ecs.worlds[CORE].increment_tick();

    if hierarchy_changed || !world.resources.transform_state.children_cache_valid {
        validate_and_rebuild_children_cache(world);
    }

    if dirty_entities.is_empty() {
        world.resources.transform_state.dirty_scratch = dirty_entities;
        return;
    }

    dirty_entities.sort_unstable_by_key(|entity| (entity.id, entity.generation));
    dirty_entities.dedup();

    let mut seen = std::mem::take(&mut world.resources.transform_state.expansion_seen);
    seen.clear();
    seen.extend(dirty_entities.iter().copied());
    {
        let children_cache = &world.resources.transform_state.children_cache;
        let mut index = 0;
        while index < dirty_entities.len() {
            let entity = dirty_entities[index];
            index += 1;
            if let Some(children) = children_cache.get(&entity) {
                for child in children {
                    if seen.insert(*child) {
                        dirty_entities.push(*child);
                    }
                }
            }
        }
    }

    let _span = tracing::info_span!("transforms", dirty_count = dirty_entities.len()).entered();

    let mut stack = std::mem::take(&mut world.resources.transform_state.traversal_stack);
    let mut render_dirty_entities =
        std::mem::take(&mut world.resources.transform_state.render_dirty_scratch);
    stack.clear();
    render_dirty_entities.clear();

    for entity in dirty_entities.iter().copied() {
        let parent_is_dirty = world.ecs.worlds[CORE]
            .get::<crate::ecs::transform::components::Parent>(entity)
            .and_then(|parent| parent.0)
            .is_some_and(|parent_entity| seen.contains(&parent_entity));
        if !parent_is_dirty {
            stack.push(entity);
        }
    }

    while let Some(current) = stack.pop() {
        if !seen.remove(&current) {
            continue;
        }

        let local_matrix = match world.ecs.worlds[CORE]
            .get::<crate::ecs::transform::components::LocalTransform>(current)
        {
            Some(local_transform) => local_transform.as_matrix(),
            None => nalgebra_glm::Mat4::identity(),
        };

        let new_global_transform = match world.ecs.worlds[CORE]
            .get::<crate::ecs::transform::components::Parent>(current)
            .and_then(|parent| parent.0)
        {
            Some(parent_entity) => {
                let mut parent_matrix = world.ecs.worlds[CORE]
                    .get::<crate::ecs::transform::components::GlobalTransform>(parent_entity)
                    .map(|global_transform| global_transform.0)
                    .unwrap_or_else(nalgebra_glm::Mat4::identity);
                if world.ecs.worlds[CORE]
                    .has::<crate::ecs::transform::components::IgnoreParentScale>(current)
                {
                    parent_matrix = remove_scale_from_matrix(parent_matrix);
                }
                parent_matrix * local_matrix
            }
            None => local_matrix,
        };

        if let Some(global_transform) =
            world.ecs.worlds[CORE]
                .get_mut::<crate::ecs::transform::components::GlobalTransform>(current)
        {
            *global_transform = GlobalTransform(new_global_transform);
        }

        let render_relevant = RENDER_MESH
            | INSTANCED_MESH
            | crate::ecs::world::LIGHT
            | crate::ecs::world::DECAL
            | crate::ecs::world::TEXT
            | crate::ecs::world::WATER
            | crate::ecs::world::CLOTH;
        if world.ecs.worlds[CORE].component_mask(current).unwrap_or(0) & render_relevant != 0 {
            render_dirty_entities.push(current);
        }

        if let Some(children) = world.resources.transform_state.children_cache.get(&current) {
            for child in children {
                stack.push(*child);
            }
        }
    }

    for entity in render_dirty_entities.drain(..) {
        world
            .resources
            .mesh_render_state
            .mark_transform_dirty(entity);
    }

    seen.clear();
    stack.clear();
    dirty_entities.clear();
    world.resources.transform_state.expansion_seen = seen;
    world.resources.transform_state.traversal_stack = stack;
    world.resources.transform_state.render_dirty_scratch = render_dirty_entities;
    world.resources.transform_state.dirty_scratch = dirty_entities;
}

fn remove_scale_from_matrix(matrix: nalgebra_glm::Mat4) -> nalgebra_glm::Mat4 {
    let mut result = matrix;

    let right = nalgebra_glm::vec3(matrix[(0, 0)], matrix[(1, 0)], matrix[(2, 0)]);
    let up = nalgebra_glm::vec3(matrix[(0, 1)], matrix[(1, 1)], matrix[(2, 1)]);
    let forward = nalgebra_glm::vec3(matrix[(0, 2)], matrix[(1, 2)], matrix[(2, 2)]);

    let right_normalized = nalgebra_glm::normalize(&right);
    let up_normalized = nalgebra_glm::normalize(&up);
    let forward_normalized = nalgebra_glm::normalize(&forward);

    result[(0, 0)] = right_normalized.x;
    result[(1, 0)] = right_normalized.y;
    result[(2, 0)] = right_normalized.z;

    result[(0, 1)] = up_normalized.x;
    result[(1, 1)] = up_normalized.y;
    result[(2, 1)] = up_normalized.z;

    result[(0, 2)] = forward_normalized.x;
    result[(1, 2)] = forward_normalized.y;
    result[(2, 2)] = forward_normalized.z;

    result
}
