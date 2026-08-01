use crate::data::generational_registry::registry_lookup_index;
use crate::data::geometry::Mesh;
use crate::resources::mesh_cache::MeshCache;

pub fn mesh_cache_holds(cache: &MeshCache, name: &str) -> bool {
    mesh_cache_mesh(cache, name).is_some()
}

pub fn mesh_cache_mesh<'cache>(cache: &'cache MeshCache, name: &str) -> Option<&'cache Mesh> {
    let (index, _) = registry_lookup_index(&cache.registry, name)?;
    cache.registry.entries.get(index as usize)?.as_ref()
}

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
