//! wgpu rendering backend.
//!
//! The [`WgpuRenderer`] supports DirectX 12, Metal, Vulkan, and WebGPU backends.
//!
//! Key types:
//!
//! - [`WgpuRenderer`]: Main renderer managing the GPU device, surface, and render graph
//! - [`CameraViewport`]: Off-screen render target for camera output
//!
//! Submodules:
//!
//! - [`rendergraph`]: Declarative pass-based render graph with automatic resource management
//! - [`passes`]: Built-in geometry, shadow, and post-processing passes
//! - [`texture_cache`]: GPU texture caching and lifecycle management
//! - [`glyph_atlas`]: dynamic glyph atlas backed by cosmic-text + swash

pub mod brdf_lut;
mod camera_viewport;
#[cfg(feature = "hdr")]
pub mod envmap_filter;
mod execution;
pub mod frame;
pub mod glyph_atlas;
#[cfg(feature = "hdr")]
pub mod hdr;
pub mod ibl;
mod initialization;
pub mod lights;
pub mod material_texture_arrays;
pub mod mip_generator;
pub mod pass_access;
pub mod pass_sync;
pub mod passes;
pub mod picking;
pub mod presentation;
pub mod render_configs;
mod surface;
pub mod texture_uploads;
pub mod view;
pub use crate::rendergraph;
pub mod shader_compose;
mod stagger;
pub mod texture_cache;
pub mod texture_array_pool;

pub use ::wgpu::*;

#[derive(Default, Clone)]
pub struct WindowRenderState {
    /// Every distinct buffer size this window has dispatched at this
    /// frame (or recently). The cache lets multi-camera-per-window
    /// dispatches early-return when the shared graph is already at the
    /// requested size for *any* of this window's cameras, instead of
    /// always falling through to the per-pass texture pool.
    pub recent_buffer_sizes: Vec<(u32, u32)>,
    pub last_settings_version: Option<u64>,
}

const RECENT_BUFFER_SIZE_CAPACITY: usize = 8;

pub struct CameraViewport {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub size: (u32, u32),
    pub has_rendered_at_least_once: bool,
    pub last_active_view: Option<crate::config::EffectiveShading>,
    pub last_settings_version: u64,
    pub last_render_frame: u64,
    pub last_camera_world_transform: Option<nalgebra_glm::Mat4>,
}

pub const DEPTH_PICK_SAMPLE_SIZE: u32 = 5;

pub struct WgpuRenderer {
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub surface_format: wgpu::TextureFormat,
    pub supported_present_modes: Vec<wgpu::PresentMode>,
    pub graph: rendergraph::RenderGraph<crate::wgpu::render_configs::RenderInputs>,
    /// Render-graph resource handles for the built-in pipeline targets.
    pub targets: RenderTargets,
    /// Persistent spotlight shadow atlas, kept across frames so cached shadow
    /// slots survive. Fed to the render graph as an external resource each frame.
    pub spotlight_shadow_atlas_texture: wgpu::Texture,
    pub spotlight_shadow_atlas_view: wgpu::TextureView,
    pub ui_image_pass: Option<Box<passes::UiImagePass>>,
    pub glyph_atlas: glyph_atlas::GlyphAtlas,
    /// Per-camera viewport texture pool.
    pub camera_viewports: std::collections::HashMap<freecs::Entity, CameraViewport>,
    pub _brdf_lut_texture: wgpu::Texture,
    pub brdf_lut_view: wgpu::TextureView,
    pub material_texture_arrays: material_texture_arrays::MaterialTextureArrays,
    pub ui_texture_array: texture_array_pool::TextureArrayPool,
    pub mip_generator: mip_generator::MipGenerator,
    /// GPU depth-pick readback state.
    pub depth_pick: DepthPickState,
    /// Screenshot readback state.
    #[cfg(not(target_arch = "wasm32"))]
    pub screenshot: ScreenshotState,
    pub render_buffer_size: (u32, u32),
    /// Render-graph state. Tracks the last buffer size used by the
    /// renderer's dispatch so consecutive dispatches don't
    /// pessimistically resize when the size is already current.
    pub window_render_state: WindowRenderState,
    /// Per-frame and dedup bookkeeping.
    pub frame_state: FrameState,
    /// The adapter the renderer selected, published into `RendererState` on the
    /// first frame so game code can adapt quality and controls to the device.
    pub gpu_profile: crate::config::GpuProfile,
    /// Image-based-lighting texture views, persisted across frames and swapped
    /// into [`RenderInputs`](render_configs::RenderInputs) for the frame.
    pub ibl_views: crate::config::IblViews,
    /// The view a meshlet cull is frozen to, captured the frame freezing turns
    /// on and held until it turns off. Held here rather than rebuilt per frame
    /// so it stays put while the camera moves away from it, which is what makes
    /// the frozen cut visible as the camera leaves the frustum that chose it.
    pub meshlet_frozen_cull_view: Option<crate::config::RenderView>,
}

/// Render-graph resource handles for the built-in pipeline targets.
pub struct RenderTargets {
    pub depth: rendergraph::ResourceId,
    pub scene_color: rendergraph::ResourceId,
    pub compute_output: rendergraph::ResourceId,
    pub aa_output: rendergraph::ResourceId,
    pub swapchain: rendergraph::ResourceId,
    pub viewport_resource: rendergraph::ResourceId,
    pub ui_depth: rendergraph::ResourceId,
    pub entity_id: rendergraph::ResourceId,
    pub view_normals: rendergraph::ResourceId,
    pub velocity: rendergraph::ResourceId,
    pub ssao_raw: rendergraph::ResourceId,
    pub ssao: rendergraph::ResourceId,
    pub ssgi_raw: rendergraph::ResourceId,
    pub ssgi: rendergraph::ResourceId,
    pub ssr_raw: rendergraph::ResourceId,
    pub ssr: rendergraph::ResourceId,
    pub spotlight_shadow_atlas: rendergraph::ResourceId,
}

/// GPU depth-pick compute and readback resources.
pub struct DepthPickState {
    pub compute_pipeline: wgpu::ComputePipeline,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub storage_buffer: wgpu::Buffer,
    pub uniform_buffer: wgpu::Buffer,
    pub staging_buffer: wgpu::Buffer,
    pub bind_group: Option<wgpu::BindGroup>,
    pub pending: bool,
    pub map_complete: std::sync::Arc<std::sync::atomic::AtomicBool>,
    pub center: (u32, u32),
    pub texture_size: (u32, u32),
    pub camera: Option<freecs::Entity>,
}

/// Screenshot capture and readback resources.
#[cfg(not(target_arch = "wasm32"))]
pub struct ScreenshotState {
    pub staging_buffer: wgpu::Buffer,
    pub pending: bool,
    pub map_complete: std::sync::Arc<std::sync::atomic::AtomicBool>,
    pub path: Option<std::path::PathBuf>,
    pub width: u32,
    pub height: u32,
    pub max_dimension: Option<u32>,
}

/// Per-frame and dedup bookkeeping that the renderer carries between frames.
pub struct FrameState {
    pub index: u64,
    pub last_settings_signature: Option<u64>,
    pub text_mesh_signatures: std::collections::HashMap<freecs::Entity, u64>,
    pub captured_ibl_atmosphere: Option<crate::config::Atmosphere>,
    pub captured_ibl_hour: f32,
    pub captured_day_night_snapshots: bool,
}

pub async fn create_wgpu_renderer<W>(
    window_handle: W,
    initial_width: u32,
    initial_height: u32,
) -> Result<WgpuRenderer, Box<dyn std::error::Error>>
where
    W: Into<wgpu::SurfaceTarget<'static>>,
{
    WgpuRenderer::new_async(window_handle, initial_width, initial_height).await
}
