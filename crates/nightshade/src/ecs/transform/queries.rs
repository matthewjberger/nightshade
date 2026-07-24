use crate::ecs::transform::components::ChildOf;
use crate::ecs::world::CORE;
use crate::ecs::world::PREFAB_SOURCE;
use crate::ecs::world::World;

/// The entity's direct children as of the last hierarchy sync. The sync
/// runs every frame in the transform pass; call
/// [`sync_hierarchy_index`](super::systems::sync_hierarchy_index) first
/// when links changed earlier in the same frame.
pub fn query_children(
    world: &World,
    entity: nightshade_ecs::Entity,
) -> Vec<nightshade_ecs::Entity> {
    world
        .res::<nightshade_ecs::dynamic::HierarchyIndex>()
        .children(entity)
        .to_vec()
}

/// The entity's whole subtree below it as of the last hierarchy sync,
/// excluding the entity itself.
pub fn query_descendants(
    world: &World,
    entity: nightshade_ecs::Entity,
) -> Vec<nightshade_ecs::Entity> {
    world
        .res::<nightshade_ecs::dynamic::HierarchyIndex>()
        .descendants(entity)
}

pub fn query_ancestors(
    world: &World,
    entity: nightshade_ecs::Entity,
) -> Vec<nightshade_ecs::Entity> {
    let mut chain = vec![entity];
    let mut current = entity;
    while let Some(ChildOf(parent)) = world.get::<nightshade_ecs::dynamic::ChildOf>(current) {
        chain.push(*parent);
        current = *parent;
    }
    chain
}

#[cfg(feature = "assets")]
pub fn find_group_root(world: &World, entity: nightshade_ecs::Entity) -> nightshade_ecs::Entity {
    let mut current = entity;
    let mut highest_prefab_root: Option<nightshade_ecs::Entity> = None;
    loop {
        if world.ecs.worlds[CORE].entity_has_components(current, PREFAB_SOURCE) {
            highest_prefab_root = Some(current);
        }
        match world.get::<nightshade_ecs::dynamic::ChildOf>(current) {
            Some(ChildOf(parent)) => current = *parent,
            _ => break,
        }
    }
    highest_prefab_root.unwrap_or(current)
}

#[cfg(not(feature = "assets"))]
pub fn find_group_root(world: &World, entity: nightshade_ecs::Entity) -> nightshade_ecs::Entity {
    let mut current = entity;
    while let Some(ChildOf(parent)) = world.get::<nightshade_ecs::dynamic::ChildOf>(current) {
        current = *parent;
    }
    current
}

pub fn chain_from_root_to_leaf(
    world: &World,
    root: nightshade_ecs::Entity,
    leaf: nightshade_ecs::Entity,
) -> Vec<nightshade_ecs::Entity> {
    if leaf == root {
        return vec![root];
    }
    let mut chain = vec![leaf];
    let mut current = leaf;
    while let Some(ChildOf(parent)) = world.get::<nightshade_ecs::dynamic::ChildOf>(current) {
        chain.push(*parent);
        if *parent == root {
            break;
        }
        current = *parent;
    }
    chain.reverse();
    chain
}
