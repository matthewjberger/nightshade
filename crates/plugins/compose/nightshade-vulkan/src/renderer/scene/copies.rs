use crate::renderer::rays::Placement;
use crate::renderer::scene::objects::GpuObject;
use std::sync::Arc;

#[derive(Clone, Copy, Debug)]
pub struct CopyBatch {
    pub parent: [[f32; 4]; 4],
    pub first_row: u32,
    pub count: u32,
    pub first_local: u32,
    pub slot: u32,
}

#[derive(Default)]
pub struct CopyRows {
    pub rows: Arc<Vec<GpuObject>>,
    pub locals: Arc<Vec<[[f32; 4]; 4]>>,
    pub placements: Arc<Vec<Placement>>,
    pub batches: Vec<CopyBatch>,
    pub work_capacity: u32,
    pub run_capacity: u32,
    pub overlay: bool,
    pub generation: u64,
    pub seen_tick: u32,
    pub waiting: bool,
    pub overlay_enabled: bool,
}

#[derive(Clone, Default)]
pub struct CopySnapshot {
    pub rows: Arc<Vec<GpuObject>>,
    pub locals: Arc<Vec<[[f32; 4]; 4]>>,
    pub generation: u64,
}

impl CopyRows {
    pub fn snapshot(&self) -> CopySnapshot {
        CopySnapshot {
            rows: self.rows.clone(),
            locals: self.locals.clone(),
            generation: self.generation,
        }
    }
}
