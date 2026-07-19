//! The mesh cache is owned by the render layer: named mesh geometry in a
//! generational registry with dirty tracking the mesh passes consume.

use crate::asset_id::MeshId;
use crate::generational_registry::*;
use crate::geometry::Mesh;
use std::collections::HashSet;

/// Named mesh geometry in a generational registry with dirty sets the mesh
/// passes consume.
#[derive(Clone, Default)]
pub struct MeshCache {
    /// Generational registry of meshes keyed by name.
    pub registry: GenerationalRegistry<Mesh>,
    /// Names of meshes changed since the last drain by the mesh pass.
    pub dirty_meshes: HashSet<String>,
    /// Names of skinned meshes changed since the last drain by the skinned mesh pass.
    pub dirty_skinned_meshes: HashSet<String>,
    /// Bumped whenever the cache is wholesale cleared. The mesh passes watch
    /// this to reset their GPU geometry registries so stale geometry from
    /// replaced models does not accumulate in the vertex and index buffers.
    pub clear_generation: u64,
}

/// Inserts or replaces the mesh under `name`, marks it dirty, and returns its id.
pub fn mesh_cache_insert(cache: &mut MeshCache, name: String, mesh: Mesh) -> MeshId {
    if let Some((index, generation)) = registry_lookup_index(&cache.registry, &name) {
        if let Some(entry) = cache.registry.entries.get_mut(index as usize) {
            *entry = Some(mesh);
        }
        cache.dirty_meshes.insert(name);
        return MeshId::new(index, generation);
    }
    let (index, generation) = registry_insert(&mut cache.registry, name.clone(), mesh);
    registry_add_reference(&mut cache.registry, index);
    cache.dirty_meshes.insert(name);
    MeshId::new(index, generation)
}

/// Drains and returns the set of dirty mesh names.
pub fn mesh_cache_take_dirty(cache: &mut MeshCache) -> HashSet<String> {
    std::mem::take(&mut cache.dirty_meshes)
}

/// Marks `name` dirty so the mesh pass re-uploads its geometry.
pub fn mesh_cache_mark_dirty(cache: &mut MeshCache, name: String) {
    cache.dirty_meshes.insert(name);
}

/// Drains and returns the set of dirty skinned mesh names.
pub fn mesh_cache_take_dirty_skinned(cache: &mut MeshCache) -> HashSet<String> {
    std::mem::take(&mut cache.dirty_skinned_meshes)
}

/// Iterates over the live meshes as `(name, mesh)` pairs.
pub fn mesh_cache_iter(cache: &MeshCache) -> impl Iterator<Item = (&String, &Mesh)> {
    cache
        .registry
        .index_to_name
        .iter()
        .enumerate()
        .filter_map(|(index, name_opt)| {
            name_opt
                .as_ref()
                .zip(cache.registry.entries[index].as_ref())
        })
}

/// Removes meshes with no remaining references, marks each removed name dirty,
/// and returns the removed names.
pub fn mesh_cache_remove_unused(cache: &mut MeshCache) -> Vec<String> {
    let removed = registry_remove_unused(&mut cache.registry);
    cache.dirty_meshes.extend(removed.iter().cloned());
    cache.dirty_skinned_meshes.extend(removed.iter().cloned());
    removed
}

/// Clears every mesh, marks all removed names dirty, and bumps `clear_generation`.
pub fn mesh_cache_clear(cache: &mut MeshCache) {
    let removed_names: Vec<String> = cache
        .registry
        .index_to_name
        .iter()
        .filter_map(|name_opt| name_opt.clone())
        .collect();
    cache.registry = GenerationalRegistry::default();
    cache.dirty_meshes.extend(removed_names.iter().cloned());
    cache.dirty_skinned_meshes.extend(removed_names);
    cache.clear_generation = cache.clear_generation.wrapping_add(1);
}
