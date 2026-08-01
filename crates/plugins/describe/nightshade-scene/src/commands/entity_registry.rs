use nightshade_ecs::Entity;

pub fn ensure_guid(
    storage: &mut nightshade_ecs::Storage,
    registry: &mut crate::resources::entity_registry::EntityRegistry,
    entity: Entity,
) -> crate::contract::primitives::Guid {
    if let Some(existing) = storage
        .get::<crate::contract::primitives::Guid>(entity)
        .copied()
    {
        registry.guid_index.entry(existing.0).or_insert(entity);
        return existing;
    }
    registry.next_guid = registry.next_guid.saturating_add(1);
    let guid = crate::contract::primitives::Guid(registry.next_guid);
    if !storage.entity_has_components::<(crate::contract::primitives::Guid,)>(entity) {
        storage.add_components::<(crate::contract::primitives::Guid,)>(entity);
    }
    storage.set(entity, guid);
    registry.guid_index.insert(guid.0, entity);
    guid
}
