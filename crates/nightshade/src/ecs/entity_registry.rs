use crate::ecs::scene::EntityBlueprint;
use crate::ecs::world::World;
use freecs::Entity;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::collections::HashMap;

#[derive(Default, Clone)]
pub struct EntityRegistry {
    pub names: HashMap<String, Entity>,
    pub tags: HashMap<Entity, Vec<String>>,
    pub blueprints: HashMap<Entity, Vec<EntityBlueprint>>,
}

/// Attach a typed game blueprint to `entity` under `kind`.
/// Overwrites any existing blueprint with the same `kind`. The game
/// owns the type `T`; the engine just stores bincode bytes.
pub fn entity_set_blueprint<T: Serialize>(
    world: &mut World,
    entity: Entity,
    kind: &str,
    value: &T,
) {
    let data = match bincode::serialize(value) {
        Ok(data) => data,
        Err(error) => {
            tracing::error!(?error, kind, "failed to serialize entity blueprint");
            return;
        }
    };
    let list = world
        .resources
        .entities
        .blueprints
        .entry(entity)
        .or_default();
    if let Some(existing) = list.iter_mut().find(|blueprint| blueprint.kind == kind) {
        existing.data = data;
    } else {
        list.push(EntityBlueprint {
            kind: kind.to_string(),
            data,
        });
    }
}

/// Returns the typed blueprint for `entity` of the given `kind`, or
/// `None` if no blueprint exists or the bytes fail to decode as `T`.
pub fn entity_get_blueprint<T: DeserializeOwned>(
    world: &World,
    entity: Entity,
    kind: &str,
) -> Option<T> {
    let list = world.resources.entities.blueprints.get(&entity)?;
    let blueprint = list.iter().find(|blueprint| blueprint.kind == kind)?;
    match bincode::deserialize(&blueprint.data) {
        Ok(value) => Some(value),
        Err(error) => {
            tracing::error!(?error, kind, "failed to deserialize entity blueprint");
            None
        }
    }
}

/// Removes the blueprint of the given `kind` from `entity`, returning
/// `true` if a blueprint was present.
pub fn entity_remove_blueprint(world: &mut World, entity: Entity, kind: &str) -> bool {
    let Some(list) = world.resources.entities.blueprints.get_mut(&entity) else {
        return false;
    };
    let Some(position) = list.iter().position(|blueprint| blueprint.kind == kind) else {
        return false;
    };
    list.swap_remove(position);
    if list.is_empty() {
        world.resources.entities.blueprints.remove(&entity);
    }
    true
}
