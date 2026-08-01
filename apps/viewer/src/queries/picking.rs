use std::collections::HashSet;

use nightshade::prelude::*;
use nightshade_scene::contract::LocalTransform;

pub fn find_root(
    storage: &nightshade::ecs::Storage,
    leaf: Entity,
    tracked: &HashSet<Entity>,
) -> Entity {
    let mut current = leaf;
    let mut steps = 0;
    while steps < 64 {
        let Some(parent) = storage
            .get::<nightshade::ecs::ChildOf>(current)
            .map(|link| link.0)
        else {
            break;
        };
        if !tracked.contains(&parent) {
            break;
        }
        current = parent;
        steps += 1;
    }
    current
}

pub fn chain_from_root(
    storage: &nightshade::ecs::Storage,
    root: Entity,
    leaf: Entity,
) -> Vec<Entity> {
    let mut chain = vec![leaf];
    let mut current = leaf;
    let mut steps = 0;
    while current != root && steps < 64 {
        let Some(parent) = storage
            .get::<nightshade::ecs::ChildOf>(current)
            .map(|link| link.0)
        else {
            break;
        };
        chain.push(parent);
        current = parent;
        steps += 1;
    }
    chain.reverse();
    chain
}

pub fn find_entity_by_id(storage: &nightshade::ecs::Storage, id: u32) -> Option<Entity> {
    storage
        .query_entities::<(LocalTransform,)>()
        .find(|entity| entity.id == id)
}
