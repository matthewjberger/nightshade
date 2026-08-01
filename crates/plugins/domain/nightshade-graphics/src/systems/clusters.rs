use crate::commands::clusters::{clusters_collect, clusters_forget, clusters_request};
use crate::queries::clusters::clusters_waiting;
use crate::resources::clusters::{CLUSTERS_IN_FLIGHT, CLUSTERS_TAKEN, ClusterCache};
use nightshade::prelude::*;
use nightshade_render::commands::mesh_cache::{
    mesh_cache_mark_dirty_clusters, mesh_cache_take_dirty_clusters,
};
use nightshade_render::queries::mesh_cache::{mesh_cache_holds, mesh_cache_mesh};
use nightshade_render::resources::mesh_cache::MeshCache;

pub fn build_the_clusters(mut meshes: ResMut<MeshCache>, mut clusters: ResMut<ClusterCache>) {
    clusters_collect(&mut clusters, CLUSTERS_TAKEN);

    let asked = mesh_cache_take_dirty_clusters(&mut meshes);
    if asked.is_empty() {
        return;
    }

    let mut later: Vec<String> = Vec::new();
    for name in asked {
        if !mesh_cache_holds(&meshes, &name) {
            clusters_forget(&mut clusters, &name);
            continue;
        }
        if clusters_waiting(&clusters) >= CLUSTERS_IN_FLIGHT {
            later.push(name);
            continue;
        }
        let Some(mesh) = mesh_cache_mesh(&meshes, &name) else {
            continue;
        };
        clusters_request(&mut clusters, &name, mesh);
    }
    for name in later {
        mesh_cache_mark_dirty_clusters(&mut meshes, name);
    }
}
