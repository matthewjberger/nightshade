use crate::ecs::world::CORE;
use crate::ecs::world::World;
use freecs::Entity;
use std::collections::HashMap;

#[derive(Default, Clone)]
pub struct EntityRegistry {
    pub names: HashMap<String, Entity>,
    pub tags: HashMap<Entity, Vec<String>>,
    pub next_guid: u64,
    pub guid_index: HashMap<u64, Entity>,
}

/// Returns the entity's `Guid`, minting a fresh value if absent and
/// adding the `GUID` archetype flag if the entity doesn't already
/// carry it. Callers that know an entity is persistent should include
/// `GUID` in the initial spawn bitflags to skip the archetype
/// migration.
pub fn ensure_guid(world: &mut World, entity: Entity) -> crate::ecs::primitives::Guid {
    if let Some(existing) = world.get::<crate::ecs::primitives::Guid>(entity).copied() {
        world
            .expect_resource_mut::<crate::ecs::entity_registry::EntityRegistry>()
            .guid_index
            .entry(existing.0)
            .or_insert(entity);
        return existing;
    }
    world
        .expect_resource_mut::<crate::ecs::entity_registry::EntityRegistry>()
        .next_guid = world
        .expect_resource_mut::<crate::ecs::entity_registry::EntityRegistry>()
        .next_guid
        .saturating_add(1);
    let guid = crate::ecs::primitives::Guid(
        world
            .expect_resource_mut::<crate::ecs::entity_registry::EntityRegistry>()
            .next_guid,
    );
    if !world.ecs.worlds[CORE].entity_has_components(entity, crate::ecs::world::GUID) {
        world.ecs.worlds[CORE].add_components(entity, crate::ecs::world::GUID);
    }
    world.set(entity, guid);
    world
        .expect_resource_mut::<crate::ecs::entity_registry::EntityRegistry>()
        .guid_index
        .insert(guid.0, entity);
    guid
}

/// Records that `entity` carries `guid` without touching the counter.
/// Used by scene load to restore authored guid values into the
/// reverse index.
pub fn assign_guid(world: &mut World, entity: Entity, guid: crate::ecs::primitives::Guid) {
    if !world.ecs.worlds[CORE].entity_has_components(entity, crate::ecs::world::GUID) {
        world.ecs.worlds[CORE].add_components(entity, crate::ecs::world::GUID);
    }
    world.set(entity, guid);
    world
        .expect_resource_mut::<crate::ecs::entity_registry::EntityRegistry>()
        .guid_index
        .insert(guid.0, entity);
    if guid.0
        > world
            .expect_resource_mut::<crate::ecs::entity_registry::EntityRegistry>()
            .next_guid
    {
        world
            .expect_resource_mut::<crate::ecs::entity_registry::EntityRegistry>()
            .next_guid = guid.0;
    }
}

pub fn find_entity_by_guid(world: &World, guid: crate::ecs::primitives::Guid) -> Option<Entity> {
    world
        .expect_resource::<crate::ecs::entity_registry::EntityRegistry>()
        .guid_index
        .get(&guid.0)
        .copied()
}
