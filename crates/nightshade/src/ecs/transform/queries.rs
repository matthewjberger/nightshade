use crate::ecs::world::{GLOBAL_TRANSFORM, LOCAL_TRANSFORM, PARENT, World, components::Parent};

pub fn query_children(world: &World, entity: freecs::Entity) -> Vec<freecs::Entity> {
    world
        .query_entities(PARENT)
        .filter(|child_entity| {
            world
                .get_parent(*child_entity)
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

pub fn query_root_nodes(world: &World) -> Vec<freecs::Entity> {
    world
        .query_entities(LOCAL_TRANSFORM | GLOBAL_TRANSFORM)
        .filter(|entity| !world.entity_has_parent(*entity))
        .collect()
}

pub fn query_reparent_candidates(world: &World, entity: freecs::Entity) -> Vec<freecs::Entity> {
    let descendants = query_descendants(world, entity);

    world
        .query_entities(LOCAL_TRANSFORM | GLOBAL_TRANSFORM)
        .filter(|candidate| *candidate != entity && !descendants.contains(candidate))
        .collect()
}
