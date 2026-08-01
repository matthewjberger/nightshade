use crate::renderer::VulkanRenderer;
use nightshade::World;
use nightshade::ecs::{ResourceMap, Storage};

pub type PassInstallFn = Box<dyn FnMut(&mut VulkanRenderer)>;

pub use crate::pre_render::PreRenderFn;

pub type WindowEventFn =
    Box<dyn FnMut(&mut Storage, &mut ResourceMap, &nightshade_platform::winit::event::WindowEvent)>;

#[derive(Default)]
pub struct RenderHooks {
    pub install: Vec<PassInstallFn>,
    pub pre_render: Vec<PreRenderFn>,
    pub window_event: Vec<WindowEventFn>,
}

#[derive(Default, Clone, Debug)]
pub struct GpuTimings {
    pub passes: Vec<PassTiming>,
    pub frame_milliseconds: f32,
}

#[derive(Default, Clone, Copy, Debug)]
pub struct GpuDrawn {
    pub clusters: u32,
    pub triangles: u32,
}

#[derive(Clone, Debug)]
pub struct PassTiming {
    pub name: &'static str,
    pub milliseconds: f32,
}

#[derive(Default)]
pub struct TextMeshCache {
    pub labels: std::collections::HashMap<nightshade::ecs::Entity, CachedLabel>,
}

pub struct CachedLabel {
    pub signature: u64,
    pub mesh: nightshade_text::contract::data::text_data::TextMesh,
}

pub struct StageState {
    pub startup: nightshade::ecs::Schedule<World>,
    pub stages: nightshade::ecs::Stages<World>,
    pub window_event: Vec<WindowEventFn>,
}

#[derive(Default)]
pub struct SpawnOffsets {
    pub offsets: std::collections::HashMap<nightshade::ecs::Entity, f32>,
}
