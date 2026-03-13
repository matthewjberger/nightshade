use crate::ecs::world::{PARENT, World, components::Parent};

pub fn query_children(world: &World, entity: freecs::Entity) -> Vec<freecs::Entity> {
    world
        .core.query_entities(PARENT)
        .filter(|child_entity| {
            world
                .core.get_parent(*child_entity)
                .is_some_and(|Parent(parent)| parent == &Some(entity))
        })
        .collect()
}

pub fn query_descendants(world: &World, entity: freecs::Entity) -> Vec<freecs::Entity> {
    let mut descendants = Vec::new();
    let children = query_children(world, entity);

    for child in children {
        descendants.push(child);
        descendants.extend(query_descendants(world, child));
    }

    descendants
}
