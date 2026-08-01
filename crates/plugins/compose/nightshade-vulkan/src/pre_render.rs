use crate::renderer::VulkanRenderer;

pub use nightshade_window::pre_render::{
    IntoPreRenderSystem, RendererExtractMarker, RendererMarker, RendererQueryMarker,
};

pub type Renderer<'frame> = nightshade_window::pre_render::RendererParam<'frame, VulkanRenderer>;

pub type PreRenderFn = nightshade_window::pre_render::PreRenderFn<VulkanRenderer>;
