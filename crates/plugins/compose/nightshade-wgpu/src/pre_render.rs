use crate::renderer::wgpu::WgpuRenderer;

pub use nightshade_window::pre_render::{
    IntoPreRenderSystem, RendererExtractMarker, RendererMarker, RendererQueryMarker,
};

pub type Renderer<'frame> = nightshade_window::pre_render::RendererParam<'frame, WgpuRenderer>;

pub type PreRenderFn = nightshade_window::pre_render::PreRenderFn<WgpuRenderer>;
