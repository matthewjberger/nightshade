use crate::resources::clusters::ClusterCache;
use nightshade_render::data::meshlet::MeshletMesh;

pub fn clusters_of<'cache>(cache: &'cache ClusterCache, name: &str) -> Option<&'cache MeshletMesh> {
    cache.built.get(name).map(std::sync::Arc::as_ref)
}

pub fn clusters_waiting(cache: &ClusterCache) -> usize {
    cache.asked.len()
}

pub fn clusters_held(cache: &ClusterCache) -> usize {
    cache.built.len()
}
