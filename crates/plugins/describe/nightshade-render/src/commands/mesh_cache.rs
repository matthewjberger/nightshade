use std::collections::HashSet;

use crate::data::asset_id::MeshId;
use crate::data::generational_registry::*;
use crate::data::geometry::Mesh;
use crate::resources::mesh_cache::MeshCache;

pub fn mesh_cache_insert(cache: &mut MeshCache, name: String, mesh: Mesh) -> MeshId {
    if let Some((index, generation)) = registry_lookup_index(&cache.registry, &name) {
        if let Some(entry) = cache.registry.entries.get_mut(index as usize) {
            *entry = Some(mesh);
        }
        cache.dirty_meshes.insert(name.clone());
        cache.dirty_clusters.insert(name.clone());
        cache.dirty_skinned_meshes.insert(name);
        return MeshId::new(index, generation);
    }
    let (index, generation) = registry_insert(&mut cache.registry, name.clone(), mesh);
    registry_add_reference(&mut cache.registry, index);
    cache.dirty_meshes.insert(name.clone());
    cache.dirty_clusters.insert(name.clone());
    cache.dirty_skinned_meshes.insert(name);
    MeshId::new(index, generation)
}

pub fn mesh_cache_protect(cache: &mut MeshCache, name: String) {
    registry_protect_name(&mut cache.registry, &name);
}

pub fn mesh_cache_release(cache: &mut MeshCache, name: &str) {
    registry_release_name(&mut cache.registry, name);
}

pub fn mesh_cache_take_dirty(cache: &mut MeshCache) -> HashSet<String> {
    std::mem::take(&mut cache.dirty_meshes)
}

pub fn mesh_cache_mark_dirty(cache: &mut MeshCache, name: String) {
    cache.dirty_meshes.insert(name);
}

pub fn mesh_cache_take_dirty_skinned(cache: &mut MeshCache) -> HashSet<String> {
    std::mem::take(&mut cache.dirty_skinned_meshes)
}

pub fn mesh_cache_take_dirty_clusters(cache: &mut MeshCache) -> HashSet<String> {
    std::mem::take(&mut cache.dirty_clusters)
}

pub fn mesh_cache_mark_dirty_clusters(cache: &mut MeshCache, name: String) {
    cache.dirty_clusters.insert(name);
}

pub fn mesh_cache_reclaim_range(
    cache: &mut MeshCache,
    live: &std::collections::HashSet<u64>,
    cursor: usize,
    budget: usize,
) -> (Vec<String>, usize) {
    crate::data::generational_registry::registry_observe(&mut cache.registry, live);
    let (removed, next) = crate::data::generational_registry::registry_reclaim_range(
        &mut cache.registry,
        live,
        cursor,
        budget,
    );
    cache.dirty_meshes.extend(removed.iter().cloned());
    cache.dirty_clusters.extend(removed.iter().cloned());
    cache.dirty_skinned_meshes.extend(removed.iter().cloned());
    (removed, next)
}

pub fn mesh_cache_clear(cache: &mut MeshCache) {
    let removed_names: Vec<String> = cache
        .registry
        .index_to_name
        .iter()
        .filter_map(|name_opt| name_opt.clone())
        .collect();
    cache.registry = GenerationalRegistry::default();
    cache.dirty_meshes.extend(removed_names.iter().cloned());
    cache.dirty_clusters.extend(removed_names.iter().cloned());
    cache.dirty_skinned_meshes.extend(removed_names);
    cache.clear_generation = cache.clear_generation.wrapping_add(1);
}
