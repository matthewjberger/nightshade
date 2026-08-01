use nightshade::ecs::Entity;

use crate::data::asset_id::TextureId;
use crate::data::generational_registry::*;
use crate::resources::texture_cache::{TextureCache, TextureOwner};

pub fn texture_cache_reserve_id(cache: &mut TextureCache, name: String) -> TextureId {
    let (index, generation) = registry_reserve_name(&mut cache.registry, name);
    TextureId::new(index, generation)
}

pub fn texture_cache_acquire(cache: &mut TextureCache, owner: TextureOwner, names: Vec<String>) {
    if let Some(previous) = cache.owners.remove(&owner) {
        for id in previous {
            registry_remove_reference(&mut cache.registry, id.index);
        }
    }
    if names.is_empty() {
        return;
    }
    let mut ids = Vec::with_capacity(names.len());
    for name in names {
        let (index, generation) = registry_reserve_name(&mut cache.registry, name);
        registry_add_reference(&mut cache.registry, index);
        ids.push(TextureId::new(index, generation));
    }
    cache.owners.insert(owner, ids);
}

pub fn texture_cache_release(cache: &mut TextureCache, owner: &TextureOwner) {
    if let Some(ids) = cache.owners.remove(owner) {
        for id in ids {
            registry_remove_reference(&mut cache.registry, id.index);
        }
    }
}

pub fn texture_cache_release_entity(cache: &mut TextureCache, entity: Entity) {
    texture_cache_release(cache, &TextureOwner::EntityMaterial(entity));
    texture_cache_release(cache, &TextureOwner::EntityVariants(entity));
}

pub fn texture_cache_release_dead_entities(
    cache: &mut TextureCache,
    keep: impl Fn(Entity) -> bool,
) {
    let dead: Vec<TextureOwner> = cache
        .owners
        .keys()
        .filter(|owner| match owner {
            TextureOwner::EntityMaterial(entity) | TextureOwner::EntityVariants(entity) => {
                !keep(*entity)
            }
            TextureOwner::Named(_) => false,
        })
        .cloned()
        .collect();
    for owner in dead {
        texture_cache_release(cache, &owner);
    }
}

pub fn texture_cache_remove_unused(cache: &mut TextureCache) -> Vec<(String, u32)> {
    let mut removed = Vec::new();
    for index in 0..cache.registry.entries.len() {
        if cache.registry.reference_counts[index] == 0 && cache.registry.entries[index].is_some() {
            if registry_is_protected(&cache.registry, index as u32) {
                continue;
            }
            if let Some(name) = cache.registry.index_to_name[index].take() {
                cache.registry.name_to_index.remove(&name);
                removed.push((name, index as u32));
            }
            cache.registry.entries[index] = None;
            cache.registry.free_indices.push(index as u32);
        }
    }
    removed
}

pub fn texture_cache_protect(cache: &mut TextureCache, name: String) {
    let (index, _) = registry_reserve_name(&mut cache.registry, name);
    registry_set_protected(&mut cache.registry, index, true);
}
