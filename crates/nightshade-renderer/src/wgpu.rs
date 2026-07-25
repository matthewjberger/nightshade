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

pub(crate) mod brdf_lut;
mod camera_viewport;
#[cfg(feature = "hdr")]
pub(crate) mod envmap_filter;
mod execution;
pub mod frame;
pub(crate) mod glyph_atlas;
#[cfg(feature = "hdr")]
pub(crate) mod hdr;
pub(crate) mod ibl;
pub mod initialization;
pub(crate) mod lights;
pub mod material_texture_arrays;
pub(crate) mod mip_generator;
pub(crate) mod pass_access;
pub mod pass_sync;
pub mod passes;
pub mod picking;
pub mod presentation;
pub mod render_configs;
mod surface;
pub mod texture_uploads;
pub(crate) mod view;
pub use crate::rendergraph;
pub mod shader_compose;
mod stagger;
pub(crate) mod texture_array_pool;
/// GPU texture registry, upload, and lifecycle tracking.
pub mod texture_cache;

pub use ::wgpu::*;

/// Render-graph dispatch state carried between frames.
#[derive(Default, Clone)]
pub(crate) struct WindowRenderState {
    /// Every distinct buffer size dispatched this frame (or recently). The
    /// cache lets multi-camera dispatches early-return when the shared graph
    /// is already at the requested size for *any* camera, instead of always
    /// falling through to the per-pass texture pool.
    pub(crate) recent_buffer_sizes: Vec<(u32, u32)>,
    /// Settings version last applied to the graph, if any.
    pub(crate) last_settings_version: Option<u64>,
}

const RECENT_BUFFER_SIZE_CAPACITY: usize = 8;

/// Off-screen render target for one camera's output.
pub struct CameraViewport {
    /// Color texture the camera renders into.
    pub texture: wgpu::Texture,
    /// View over [`texture`](Self::texture).
    pub view: wgpu::TextureView,
    /// Viewport size in pixels.
    pub size: (u32, u32),
    /// Whether this viewport has produced at least one frame.
    pub has_rendered_at_least_once: bool,
    /// Effective shading mode used on the last render, for change detection.
    pub last_active_view: Option<crate::config::EffectiveShading>,
    /// Settings version applied on the last render.
    pub last_settings_version: u64,
    /// Frame index of the last render into this viewport.
    pub last_render_frame: u64,
    /// Camera world transform captured on the last render, for reuse detection.
    pub last_camera_world_transform: Option<nalgebra_glm::Mat4>,
    /// A caller-owned depth image and array layer to publish this camera's depth
    /// into after it renders, for hosts that submit depth alongside color.
    pub depth_target: Option<(wgpu::Texture, u32)>,
    /// The unjittered view-projection this camera last rendered with.
    ///
    /// Reprojection needs the previous frame *of this camera*. A frame that
    /// dispatches several cameras has several previous frames, and taking one of
    /// them globally reprojects the others against a viewpoint they never had.
    pub last_view_projection: Option<[[f32; 4]; 4]>,
}

/// Side length in texels of the square depth-pick sample window.
pub const DEPTH_PICK_SAMPLE_SIZE: u32 = 5;

/// wgpu-backed renderer owning the GPU device, surface, and render graph.
pub struct WgpuRenderer {
    /// Swapchain surface the final image is presented to.
    pub surface: wgpu::Surface<'static>,
    /// GPU device that creates all resources.
    pub device: wgpu::Device,
    /// Command queue submitting work and buffer writes to the GPU.
    pub queue: wgpu::Queue,
    /// Current swapchain configuration covering size, format, and present mode.
    pub surface_config: wgpu::SurfaceConfiguration,
    /// Texture format the surface presents in.
    pub surface_format: wgpu::TextureFormat,
    /// Present modes the surface reports as available.
    pub supported_present_modes: Vec<wgpu::PresentMode>,
    /// Render graph driving the built-in and custom passes each frame.
    pub graph: rendergraph::RenderGraph<crate::wgpu::render_configs::RenderInputs>,
    /// Render-graph resource handles for the built-in pipeline targets.
    pub targets: RenderTargets,
    /// Persistent spotlight shadow atlas, kept across frames so cached shadow
    /// slots survive. Fed to the render graph as an external resource each frame.
    pub spotlight_shadow_atlas_texture: wgpu::Texture,
    /// View over [`spotlight_shadow_atlas_texture`](Self::spotlight_shadow_atlas_texture).
    pub spotlight_shadow_atlas_view: wgpu::TextureView,
    /// UI image pass, present when retained-UI images are drawn.
    pub ui_image_pass: Option<Box<passes::PaintImagePass>>,
    /// Dynamic glyph atlas backing text rendering. Renderer-internal: drive text
    /// meshing through [`crate::wgpu::pass_sync`] rather than reaching in here.
    pub(crate) glyph_atlas: glyph_atlas::GlyphAtlas,
    /// Per-camera viewport texture pool.
    pub camera_viewports: std::collections::HashMap<nightshade_ecs::Entity, CameraViewport>,
    /// BRDF integration lookup texture, retained to keep [`brdf_lut_view`](Self::brdf_lut_view) alive.
    pub _brdf_lut_texture: wgpu::Texture,
    /// View over the BRDF integration lookup texture sampled during shading.
    pub brdf_lut_view: wgpu::TextureView,
    /// Bindless material texture arrays shared across mesh passes.
    pub material_texture_arrays: material_texture_arrays::MaterialTextureArrays,
    /// Texture array pool backing UI image draws.
    pub ui_texture_array: texture_array_pool::TextureArrayPool,
    /// Compute helper that generates mip chains for uploaded textures.
    pub mip_generator: mip_generator::MipGenerator,
    /// GPU depth-pick readback state. Renderer-internal: poll picks through
    /// [`crate::wgpu::picking::poll_depth_pick`] rather than reaching in here.
    pub(crate) depth_pick: DepthPickState,
    /// Screenshot readback state.
    #[cfg(all(not(target_arch = "wasm32"), feature = "screenshot"))]
    pub(crate) screenshot: ScreenshotState,
    /// Current internal render buffer size in pixels.
    pub render_buffer_size: (u32, u32),
    /// Render-graph state. Tracks the last buffer size used by the
    /// renderer's dispatch so consecutive dispatches don't
    /// pessimistically resize when the size is already current.
    pub(crate) window_render_state: WindowRenderState,
    /// Per-frame and dedup bookkeeping. Renderer-internal.
    pub(crate) frame_state: FrameState,
    /// The renderer-owned store of live GPU textures, keyed by name. The host's
    /// [`TextureCache`](crate::wgpu::texture_cache::TextureCache) holds only the
    /// CPU id/refcount registry; the wgpu handles live here and are swapped into
    /// the frame's [`RenderInputs`](render_configs::RenderInputs) so the passes
    /// can sample them.
    pub(crate) texture_store: crate::wgpu::texture_cache::TextureStore,
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
    /// Depth buffer target, reverse-Z.
    pub depth: rendergraph::ResourceId,
    /// HDR scene color target.
    pub scene_color: rendergraph::ResourceId,
    /// Compute pass output target.
    pub compute_output: rendergraph::ResourceId,
    /// Antialiasing resolve output target.
    pub aa_output: rendergraph::ResourceId,
    /// Final swapchain output target.
    pub swapchain: rendergraph::ResourceId,
    /// External viewport texture the frame renders into.
    pub viewport_resource: rendergraph::ResourceId,
    /// Depth target for UI passes.
    pub ui_depth: rendergraph::ResourceId,
    /// Per-pixel entity id target used for picking.
    pub entity_id: rendergraph::ResourceId,
    /// View-space normals target consumed by SSAO and SSGI.
    pub view_normals: rendergraph::ResourceId,
    /// Screen-space velocity target for motion-based effects.
    pub velocity: rendergraph::ResourceId,
    /// Unblurred SSAO output.
    pub ssao_raw: rendergraph::ResourceId,
    /// Blurred SSAO ready for compositing.
    pub ssao: rendergraph::ResourceId,
    /// Unblurred SSGI output.
    pub ssgi_raw: rendergraph::ResourceId,
    /// Blurred SSGI ready for compositing.
    pub ssgi: rendergraph::ResourceId,
    /// Unblurred screen-space reflection output.
    pub ssr_raw: rendergraph::ResourceId,
    /// Resolved screen-space reflection output.
    pub ssr: rendergraph::ResourceId,
    /// External spotlight shadow atlas fed into the graph each frame.
    pub spotlight_shadow_atlas: rendergraph::ResourceId,
}

/// GPU depth-pick compute and readback resources.
pub(crate) struct DepthPickState {
    /// Compute pipeline that samples depth around the pick center.
    pub compute_pipeline: wgpu::ComputePipeline,
    /// Layout for the depth-pick bind group.
    pub bind_group_layout: wgpu::BindGroupLayout,
    /// Storage buffer the compute shader writes sampled depths into.
    pub storage_buffer: wgpu::Buffer,
    /// Uniform buffer holding the pick parameters.
    pub uniform_buffer: wgpu::Buffer,
    /// Mappable buffer that receives the depth readback.
    pub staging_buffer: wgpu::Buffer,
    /// Cached bind group, rebuilt when the depth texture changes.
    pub bind_group: Option<wgpu::BindGroup>,
    /// Whether a readback is in flight.
    pub pending: bool,
    /// Set from the map callback when the staging buffer is ready to read.
    pub map_complete: std::sync::Arc<std::sync::atomic::AtomicBool>,
    /// Pixel center of the pick sample window.
    pub center: (u32, u32),
    /// Size of the depth texture the pick sampled.
    pub texture_size: (u32, u32),
    /// Camera the pending pick belongs to, if any.
    pub camera: Option<nightshade_ecs::Entity>,
}

/// Screenshot capture and readback resources.
#[cfg(all(not(target_arch = "wasm32"), feature = "screenshot"))]
pub(crate) struct ScreenshotState {
    /// Mappable buffer that receives the captured frame.
    pub staging_buffer: wgpu::Buffer,
    /// Whether a capture is in flight.
    pub pending: bool,
    /// Set from the map callback when the staging buffer is ready to read.
    pub map_complete: std::sync::Arc<std::sync::atomic::AtomicBool>,
    /// Destination path the capture is written to, if any.
    pub path: Option<std::path::PathBuf>,
    /// Captured image width in pixels.
    pub width: u32,
    /// Captured image height in pixels.
    pub height: u32,
    /// Optional cap on the larger output dimension, downscaling above it.
    pub max_dimension: Option<u32>,
}

/// Per-frame and dedup bookkeeping that the renderer carries between frames.
pub(crate) struct FrameState {
    /// Monotonic frame counter.
    pub index: u64,
    /// Settings signature applied on the previous frame, for change detection.
    pub last_settings_signature: Option<u64>,
    /// Per-entity text mesh signatures, used to skip unchanged rebuilds.
    pub text_mesh_signatures: std::collections::HashMap<nightshade_ecs::Entity, u64>,
    /// Atmosphere captured for the current IBL bake, if any.
    #[cfg(feature = "hdr")]
    pub captured_ibl_atmosphere: Option<crate::config::Atmosphere>,
    /// Time of day captured for the current IBL bake.
    #[cfg(feature = "hdr")]
    pub captured_ibl_hour: f32,
    /// Whether day-night IBL snapshots have been captured.
    #[cfg(feature = "hdr")]
    pub captured_day_night_snapshots: bool,
}

/// Creates a [`WgpuRenderer`] for `window_handle` at the given initial size.
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
