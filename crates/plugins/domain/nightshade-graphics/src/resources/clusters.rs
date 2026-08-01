use nightshade_render::data::geometry::Mesh;
use nightshade_render::data::meshlet::MeshletMesh;
use nightshade_render::queries::meshlets::build_meshlets;
use nightshade_worker::{Job, Workers, workers_hire};
use std::collections::HashMap;
use std::sync::Arc;

pub const CLUSTER_CREW: usize = 16;

pub const CLUSTERS_IN_FLIGHT: usize = 48;

pub const CLUSTERS_TAKEN: usize = 24;

pub struct ClusterJob {
    pub(crate) ticket: u64,
    pub(crate) name: String,
    pub(crate) mesh: Mesh,
}

pub struct ClusterResult {
    pub ticket: u64,
    pub name: String,
    pub clusters: Arc<MeshletMesh>,
}

impl Job for ClusterJob {
    type Key = u64;
    type Done = ClusterResult;
    type Tools = ();

    fn key(&self) -> Self::Key {
        self.ticket
    }

    fn run(self, _tools: &mut Self::Tools) -> Self::Done {
        ClusterResult {
            ticket: self.ticket,
            clusters: Arc::new(build_meshlets(&self.mesh)),
            name: self.name,
        }
    }
}

pub struct ClusterCache {
    pub(crate) crew: Workers<ClusterJob>,
    pub(crate) built: HashMap<String, Arc<MeshletMesh>>,
    pub(crate) asked: HashMap<String, u64>,
    pub(crate) next_ticket: u64,
}

impl Default for ClusterCache {
    fn default() -> Self {
        Self {
            crew: workers_hire::<ClusterJob>(CLUSTER_CREW),
            built: HashMap::new(),
            asked: HashMap::new(),
            next_ticket: 0,
        }
    }
}
