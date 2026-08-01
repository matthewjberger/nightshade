use crate::resources::clusters::{ClusterCache, ClusterJob};
use nightshade_render::data::geometry::Mesh;
use nightshade_worker::{workers_submit, workers_take};

pub fn clusters_request(cache: &mut ClusterCache, name: &str, mesh: &Mesh) -> bool {
    if cache.asked.contains_key(name) {
        return false;
    }
    cache.next_ticket += 1;
    let ticket = cache.next_ticket;
    cache.asked.insert(name.to_string(), ticket);
    workers_submit(
        &mut cache.crew,
        ClusterJob {
            ticket,
            name: name.to_string(),
            mesh: mesh.clone(),
        },
        0,
    )
}

pub fn clusters_collect(cache: &mut ClusterCache, limit: usize) -> usize {
    let done = workers_take(&mut cache.crew, limit);
    let mut kept = 0;
    for result in done {
        if cache.asked.get(&result.name) != Some(&result.ticket) {
            continue;
        }
        cache.asked.remove(&result.name);
        cache.built.insert(result.name, result.clusters);
        kept += 1;
    }
    kept
}

pub fn clusters_forget(cache: &mut ClusterCache, name: &str) {
    cache.built.remove(name);
    cache.asked.remove(name);
}

pub fn clusters_clear(cache: &mut ClusterCache) {
    cache.built.clear();
    cache.asked.clear();
}
