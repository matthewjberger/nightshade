use nightshade::ecs::Entity;
use nightshade_wgpu::wgpu;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

pub struct Parts {
    pub(crate) clear: wgpu::ComputePipeline,
    pub(crate) fill: wgpu::ComputePipeline,
    pub(crate) answer: wgpu::ComputePipeline,
    pub(crate) push: wgpu::ComputePipeline,
    pub(crate) settings: wgpu::Buffer,
    pub(crate) points: wgpu::Buffer,
    pub(crate) weights: wgpu::Buffer,
    pub(crate) asks: wgpu::Buffer,
    pub(crate) answers: wgpu::Buffer,
    pub(crate) pushes: wgpu::Buffer,
    pub(crate) staging: wgpu::Buffer,
    pub(crate) bind_group: wgpu::BindGroup,
    pub(crate) cells: u32,
    pub(crate) capacity: u32,
}

#[derive(Default)]
pub struct NearGpu {
    pub(crate) parts: Option<Parts>,
    pub(crate) ready: Arc<AtomicBool>,
    pub(crate) pending: bool,
    pub(crate) held: Vec<Entity>,
    pub(crate) tags: Vec<u32>,
    pub(crate) packed_points: Vec<[f32; 4]>,
    pub(crate) packed_weights: Vec<f32>,
    pub(crate) packed_asks: Vec<[f32; 4]>,
}
