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
