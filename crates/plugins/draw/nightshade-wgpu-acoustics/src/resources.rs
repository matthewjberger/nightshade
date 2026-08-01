use nightshade::ecs::Entity;
use nightshade_acoustics::contract::{EmitterVoice, SoundPath};
use nightshade_wgpu::wgpu;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

pub struct Parts {
    pub(crate) pipeline: wgpu::ComputePipeline,
    pub(crate) settings: wgpu::Buffer,
    pub(crate) emitters: wgpu::Buffer,
    pub(crate) cells: wgpu::Buffer,
    pub(crate) tally: wgpu::Buffer,
    pub(crate) echoes: wgpu::Buffer,
    pub(crate) steps: wgpu::Buffer,
    pub(crate) materials: wgpu::Buffer,
    pub(crate) staging: wgpu::Buffer,
    pub(crate) bind_group: wgpu::BindGroup,
    pub(crate) cell_words: usize,
}

#[derive(Default)]
pub struct AcousticsGpu {
    pub(crate) parts: Option<Parts>,
    pub(crate) packed: Vec<u8>,
    pub(crate) listened: Vec<Entity>,
    pub(crate) voices: Vec<EmitterVoice>,
    pub(crate) travels: Vec<f32>,
    pub(crate) ready: Arc<AtomicBool>,
    pub(crate) pending: bool,
    pub(crate) slice: u32,
    pub(crate) anchor: nalgebra_glm::Vec3,
    pub(crate) revision: u64,
    pub(crate) counted: [u32; 5],
    pub(crate) zone_counted: Vec<[u32; 5]>,
    pub(crate) caps: [f32; 4],
    pub(crate) loudest: f32,
    pub(crate) holds: Vec<bool>,
    pub(crate) listens: Vec<bool>,
    pub(crate) outside: Vec<bool>,
    pub(crate) beyond: f32,
    pub(crate) leeches: Vec<(Entity, Entity)>,
    pub(crate) held: Vec<(Entity, SoundPath)>,
    pub(crate) zoned: Vec<Entity>,
    pub(crate) zone_rays: u32,
    pub(crate) parked: Vec<(Entity, SoundPath)>,
    pub(crate) buckets: Vec<f32>,
    pub(crate) materials_written: bool,
}
