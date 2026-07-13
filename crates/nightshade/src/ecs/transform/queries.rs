use crate::ecs::world::CORE;
use crate::ecs::world::PREFAB_SOURCE;
use crate::ecs::world::{World, components::ChildOf};

/// The entity's direct children as of the last hierarchy sync. The sync
/// runs every frame in the transform pass; call
/// [`sync_hierarchy_index`](super::systems::sync_hierarchy_index) first
/// when links changed earlier in the same frame.
pub fn query_children(world: &World, entity: freecs::Entity) -> Vec<freecs::Entity> {
    world.resources.hierarchy.children(entity).to_vec()
}

/// The entity's whole subtree below it as of the last hierarchy sync,
/// excluding the entity itself.
pub fn query_descendants(world: &World, entity: freecs::Entity) -> Vec<freecs::Entity> {
    world.resources.hierarchy.descendants(entity)
}

pub fn query_ancestors(world: &World, entity: freecs::Entity) -> Vec<freecs::Entity> {
    let mut chain = vec![entity];
    let mut current = entity;
    while let Some(ChildOf(parent)) =
        world.ecs.worlds[CORE].get::<freecs::dynamic::ChildOf>(current)
    {
        chain.push(*parent);
        current = *parent;
    }
    chain
}

#[cfg(feature = "assets")]
pub fn find_group_root(world: &World, entity: freecs::Entity) -> freecs::Entity {
    let mut current = entity;
    let mut highest_prefab_root: Option<freecs::Entity> = None;
    loop {
        if world.ecs.worlds[CORE].entity_has_components(current, PREFAB_SOURCE) {
            highest_prefab_root = Some(current);
        }
        match world.ecs.worlds[CORE].get::<freecs::dynamic::ChildOf>(current) {
            Some(ChildOf(parent)) => current = *parent,
            _ => break,
        }
    }
    highest_prefab_root.unwrap_or(current)
}

#[cfg(not(feature = "assets"))]
pub fn find_group_root(world: &World, entity: freecs::Entity) -> freecs::Entity {
    let mut current = entity;
    while let Some(ChildOf(parent)) =
        world.ecs.worlds[CORE].get::<freecs::dynamic::ChildOf>(current)
    {
        current = *parent;
    }
    current
}

pub fn chain_from_root_to_leaf(
    world: &World,
    root: freecs::Entity,
    leaf: freecs::Entity,
) -> Vec<freecs::Entity> {
    if leaf == root {
        return vec![root];
    }
    let mut chain = vec![leaf];
    let mut current = leaf;
    while let Some(ChildOf(parent)) =
        world.ecs.worlds[CORE].get::<freecs::dynamic::ChildOf>(current)
    {
        chain.push(*parent);
        if *parent == root {
            break;
        }
        current = *parent;
    }
    chain.reverse();
    chain
}
