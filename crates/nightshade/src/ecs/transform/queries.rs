#[cfg(feature = "assets")]
use crate::ecs::world::PREFAB_SOURCE;
use crate::ecs::world::{PARENT, World, components::Parent};
use std::collections::HashMap;

pub fn query_children(world: &World, entity: freecs::Entity) -> Vec<freecs::Entity> {
    world
        .core
        .query_entities(PARENT)
        .filter(|child_entity| {
            world
                .core
                .get_parent(*child_entity)
                .is_some_and(|Parent(parent)| parent == &Some(entity))
        })
        .collect()
}

pub fn query_descendants(world: &World, entity: freecs::Entity) -> Vec<freecs::Entity> {
    let mut children_by_parent: HashMap<u32, Vec<freecs::Entity>> = HashMap::new();
    for child in world.core.query_entities(PARENT) {
        if let Some(Parent(Some(parent))) = world.core.get_parent(child) {
            children_by_parent.entry(parent.id).or_default().push(child);
        }
    }

    let mut descendants = Vec::new();
    let mut stack: Vec<freecs::Entity> = children_by_parent
        .get(&entity.id)
        .cloned()
        .unwrap_or_default();
    while let Some(current) = stack.pop() {
        descendants.push(current);
        if let Some(children) = children_by_parent.get(&current.id) {
            stack.extend(children.iter().copied());
        }
    }
    descendants
}

pub fn query_ancestors(world: &World, entity: freecs::Entity) -> Vec<freecs::Entity> {
    let mut chain = vec![entity];
    let mut current = entity;
    while let Some(Parent(Some(parent))) = world.core.get_parent(current) {
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
        if world.core.entity_has_components(current, PREFAB_SOURCE) {
            highest_prefab_root = Some(current);
        }
        match world.core.get_parent(current) {
            Some(Parent(Some(parent))) => current = *parent,
            _ => break,
        }
    }
    highest_prefab_root.unwrap_or(current)
}

#[cfg(not(feature = "assets"))]
pub fn find_group_root(world: &World, entity: freecs::Entity) -> freecs::Entity {
    let mut current = entity;
    while let Some(Parent(Some(parent))) = world.core.get_parent(current) {
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
    while let Some(Parent(Some(parent))) = world.core.get_parent(current) {
        chain.push(*parent);
        if *parent == root {
            break;
        }
        current = *parent;
    }
    chain.reverse();
    chain
}
