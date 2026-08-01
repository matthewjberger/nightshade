use std::collections::HashSet;
use std::sync::Arc;

use nightshade_worker::{Job, Workers, workers_hire};

use crate::contract::render_data::{SdfChunkKey, SdfCollisionMesh, SdfEditInstance};
use crate::data::collision::{COLLISION_CELL, COLLISION_CREW};
use crate::data::surface::chunk_surface;

pub struct SurfaceJob {
    pub(crate) key: SdfChunkKey,
    pub(crate) cell: f32,
    pub(crate) stamp: u64,
    pub(crate) edits: Arc<Vec<SdfEditInstance>>,
}

pub struct SurfaceResult {
    pub key: SdfChunkKey,
    pub stamp: u64,
    pub mesh: SdfCollisionMesh,
}

impl Job for SurfaceJob {
    type Key = SdfChunkKey;
    type Done = SurfaceResult;
    type Tools = ();

    fn key(&self) -> Self::Key {
        self.key
    }

    fn run(self, _tools: &mut Self::Tools) -> Self::Done {
        SurfaceResult {
            key: self.key,
            stamp: self.stamp,
            mesh: chunk_surface(&self.edits, self.key, self.cell),
        }
    }
}

pub struct SdfCollision {
    pub cell: f32,
    pub ready: Vec<(SdfChunkKey, SdfCollisionMesh)>,
    pub gone: Vec<SdfChunkKey>,
    pub(crate) crew: Workers<SurfaceJob>,
    pub(crate) built: HashSet<SdfChunkKey>,
    pub(crate) stale: HashSet<SdfChunkKey>,
    pub(crate) pending: HashSet<SdfChunkKey>,
    pub(crate) edits: Arc<Vec<SdfEditInstance>>,
    pub(crate) stamp: u64,
}

impl Default for SdfCollision {
    fn default() -> Self {
        Self {
            cell: COLLISION_CELL,
            ready: Vec::new(),
            gone: Vec::new(),
            crew: workers_hire::<SurfaceJob>(COLLISION_CREW),
            built: HashSet::new(),
            stale: HashSet::new(),
            pending: HashSet::new(),
            edits: Arc::new(Vec::new()),
            stamp: 0,
        }
    }
}
