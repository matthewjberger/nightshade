pub use crate::ecs::transform::queries::*;

use crate::ecs::transform::components::{ChildOf, GlobalTransform};
use crate::ecs::world::{GLOBAL_TRANSFORM, LOCAL_TRANSFORM, PARENT, World};

use crate::prelude::*;
pub fn run_systems(world: &mut World) {
    let _span = tracing::info_span!("transform_systems").entered();
    update_global_transforms_system(world);
}

/// Pull-syncs the maintained hierarchy index from the core world's
/// structural log and change ticks. Runs at the top of the transform pass
/// every frame; call it directly before reading
/// `world.res::<nightshade_ecs::dynamic::HierarchyIndex>()` when links may have changed this frame.
/// Must run before the render sync trims the structural log, so never gate
/// the transform-pass sync behind a condition.
pub fn sync_hierarchy_index(world: &mut World) {
    let mut hierarchy = std::mem::take(world.res_mut::<nightshade_ecs::dynamic::HierarchyIndex>());
    hierarchy.sync(&mut world.ecs.worlds[CORE]);
    *world.res_mut::<nightshade_ecs::dynamic::HierarchyIndex>() = hierarchy;
}

pub fn update_parent(world: &mut World, child: nightshade_ecs::Entity, new_parent: Option<Entity>) {
    match new_parent {
        Some(parent) => world.set(child, ChildOf(parent)),
        None => {
            world.ecs.worlds[CORE].remove_components(child, PARENT);
        }
    }
}

/// Recomputes [`GlobalTransform`] for every entity whose [`LocalTransform`] or
/// [`ChildOf`] changed since the previous run, plus all of their descendants.
/// Change detection rides on the ECS change ticks, so any write through the
/// component accessors is picked up automatically. The system keeps its own
/// change cursor and fences the world tick after consuming, so writes made
/// later in the frame land in the next run regardless of how often other
/// consumers advance the tick. Propagation walks each dirty subtree top down,
/// multiplying the parent's already-updated global by the child's local, one
/// matrix product per entity.
pub fn update_global_transforms_system(world: &mut World) {
    sync_hierarchy_index(world);

    let initial_pass = !world
        .res::<crate::ecs::transform::resources::TransformState>()
        .initial_pass_done;

    let mut dirty_entities = std::mem::take(
        &mut world
            .res_mut::<crate::ecs::transform::resources::TransformState>()
            .dirty_scratch,
    );
    dirty_entities.clear();

    if initial_pass {
        dirty_entities
            .extend(world.ecs.worlds[CORE].query_entities(LOCAL_TRANSFORM | GLOBAL_TRANSFORM));
        world
            .res_mut::<crate::ecs::transform::resources::TransformState>()
            .initial_pass_done = true;
    } else {
        let since_tick = world
            .res::<crate::ecs::transform::resources::TransformState>()
            .last_consumed_tick;
        dirty_entities.extend(
            world.ecs.worlds[CORE].query_entities_changed_since(LOCAL_TRANSFORM, since_tick),
        );
        dirty_entities
            .extend(world.ecs.worlds[CORE].query_entities_changed_since(PARENT, since_tick));
    }
    world
        .res_mut::<crate::ecs::transform::resources::TransformState>()
        .last_consumed_tick = world.ecs.worlds[CORE].current_tick();
    world.ecs.worlds[CORE].increment_tick();

    if dirty_entities.is_empty() {
        world
            .res_mut::<crate::ecs::transform::resources::TransformState>()
            .dirty_scratch = dirty_entities;
        return;
    }

    dirty_entities.sort_unstable_by_key(|entity| (entity.id, entity.generation));
    dirty_entities.dedup();

    let mut seen = std::mem::take(
        &mut world
            .res_mut::<crate::ecs::transform::resources::TransformState>()
            .expansion_seen,
    );
    seen.clear();
    seen.extend(dirty_entities.iter().copied());
    {
        let hierarchy = &world.res::<nightshade_ecs::dynamic::HierarchyIndex>();
        let mut index = 0;
        while index < dirty_entities.len() {
            let entity = dirty_entities[index];
            index += 1;
            for child in hierarchy.children(entity) {
                if seen.insert(*child) {
                    dirty_entities.push(*child);
                }
            }
        }
    }

    let _span = tracing::info_span!("transforms", dirty_count = dirty_entities.len()).entered();

    let mut stack = std::mem::take(
        &mut world
            .res_mut::<crate::ecs::transform::resources::TransformState>()
            .traversal_stack,
    );
    let mut render_dirty_entities = std::mem::take(
        &mut world
            .res_mut::<crate::ecs::transform::resources::TransformState>()
            .render_dirty_scratch,
    );
    stack.clear();
    render_dirty_entities.clear();

    for entity in dirty_entities.iter().copied() {
        let parent_is_dirty = world
            .get::<nightshade_ecs::dynamic::ChildOf>(entity)
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

        let new_global_transform = computed_global_transform(world, current);
        if let Some(global_transform) =
            world.get_mut::<crate::ecs::transform::components::GlobalTransform>(current)
        {
            *global_transform = GlobalTransform(new_global_transform);
        }

        // Every entity whose global transform was recomputed is recorded here;
        // the render-sync layer (`collect_scene_dirt`) filters this set by the
        // render-relevant component masks it owns. Transform propagation stays
        // ignorant of what is renderable, so it names no render-domain mask.
        render_dirty_entities.push(current);

        for child in world
            .res::<nightshade_ecs::dynamic::HierarchyIndex>()
            .children(current)
        {
            stack.push(*child);
        }
    }

    for entity in render_dirty_entities.drain(..) {
        world
            .res_mut::<crate::render::mesh_state::MeshRenderState>()
            .mark_transform_dirty(entity);
    }

    seen.clear();
    stack.clear();
    dirty_entities.clear();
    world
        .res_mut::<crate::ecs::transform::resources::TransformState>()
        .expansion_seen = seen;
    world
        .res_mut::<crate::ecs::transform::resources::TransformState>()
        .traversal_stack = stack;
    world
        .res_mut::<crate::ecs::transform::resources::TransformState>()
        .render_dirty_scratch = render_dirty_entities;
    world
        .res_mut::<crate::ecs::transform::resources::TransformState>()
        .dirty_scratch = dirty_entities;
}

fn computed_global_transform(world: &World, current: Entity) -> nalgebra_glm::Mat4 {
    let local_matrix = match world.get::<crate::ecs::transform::components::LocalTransform>(current)
    {
        Some(local_transform) => local_transform.as_matrix(),
        None => nalgebra_glm::Mat4::identity(),
    };

    match world
        .get::<nightshade_ecs::dynamic::ChildOf>(current)
        .map(|child_of| child_of.0)
    {
        Some(parent_entity) => {
            let mut parent_matrix = world
                .get::<crate::ecs::transform::components::GlobalTransform>(parent_entity)
                .map(|global_transform| global_transform.0)
                .unwrap_or_else(nalgebra_glm::Mat4::identity);
            if world.has::<crate::ecs::transform::components::IgnoreParentScale>(current) {
                parent_matrix = remove_scale_from_matrix(parent_matrix);
            }
            parent_matrix * local_matrix
        }
        None => local_matrix,
    }
}

/// Computes [`GlobalTransform`] for `root` and every descendant right now,
/// top down, so a freshly spawned subtree is world-valid the moment the
/// spawn call returns instead of after the frame schedule's propagation
/// pass. The scheduled pass still owns render dirt and change-tick
/// consumption; this walk only writes the matrices. The spawn paths call
/// it themselves; reach for it directly only after moving a subtree
/// mid-frame and needing its world transforms in the same breath.
pub fn propagate_subtree_global_transforms(world: &mut World, root: Entity) {
    sync_hierarchy_index(world);
    let mut stack = vec![root];
    while let Some(current) = stack.pop() {
        let new_global_transform = computed_global_transform(world, current);
        if let Some(global_transform) =
            world.get_mut::<crate::ecs::transform::components::GlobalTransform>(current)
        {
            *global_transform = GlobalTransform(new_global_transform);
        }
        for child in world
            .res::<nightshade_ecs::dynamic::HierarchyIndex>()
            .children(current)
        {
            stack.push(*child);
        }
    }
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

/// Sets an entity's local transform, defaults its global transform, and
/// propagates the subtree so children inherit it immediately.
pub fn setup_entity_transforms(
    world: &mut World,
    entity: Entity,
    local_transform: crate::ecs::transform::components::LocalTransform,
) {
    world.set(entity, local_transform);
    world.set(
        entity,
        crate::ecs::transform::components::GlobalTransform::default(),
    );
    propagate_subtree_global_transforms(world, entity);
}
