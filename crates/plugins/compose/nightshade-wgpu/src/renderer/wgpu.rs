use crate::renderer::wgpu::shader_compose::ShaderComposer;
pub(crate) mod brdf_lut;
pub mod build;
mod camera_viewport;
pub mod caps;
mod execution;
pub mod frame;
pub(crate) mod glyph_atlas;
pub mod initialization;
pub mod install;
pub(crate) mod lights;
pub mod material_texture_arrays;
pub(crate) mod mip_generator;
mod paint_target;
pub(crate) mod pass_access;
pub mod pass_sync;
pub mod passes;
pub mod picking;
pub mod presentation;
pub mod render_configs;
mod surface;
pub mod texture_uploads;
pub mod timing;
pub(crate) mod view;
pub use crate::renderer::create;
pub use crate::renderer::rendergraph;
pub mod pass_build;
pub mod placeholder;
pub mod shader_compose;
pub mod stagger;
pub mod texture_array_pool;
pub mod texture_cache;

pub use ::wgpu::*;

#[derive(Default, Clone)]
pub(crate) struct WindowRenderState {
    pub(crate) recent_buffer_sizes: Vec<(u32, u32)>,
    pub(crate) last_settings_version: Option<u64>,
}

const RECENT_BUFFER_SIZE_CAPACITY: usize = 8;

pub struct CameraViewport {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub size: (u32, u32),
    pub has_rendered_at_least_once: bool,
    pub last_active_view: Option<crate::renderer::config::EffectiveShading>,
    pub last_settings_version: u64,
    pub last_render_frame: u64,
    pub last_camera_world_transform: Option<nalgebra_glm::Mat4>,
    pub depth_target: Option<(wgpu::Texture, u32)>,
    pub last_view_projection: Option<[[f32; 4]; 4]>,
}

pub struct PaintTarget {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub size: (u32, u32),
}

pub const DEPTH_PICK_SAMPLE_SIZE: u32 = 5;

pub const HDR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

pub struct WgpuRenderer {
    pub surface: Option<wgpu::Surface<'static>>,
    pub instance: Option<wgpu::Instance>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub surface_format: wgpu::TextureFormat,
    pub point_shadow_format: wgpu::TextureFormat,
    pub material_texture_arrays: material_texture_arrays::MaterialTextureArrays,
    pub supported_present_modes: Vec<wgpu::PresentMode>,
    pub graph: rendergraph::RenderGraph<crate::renderer::wgpu::render_configs::RenderInputs>,
    pub targets: RenderTargets,
    pub(crate) glyph_atlas: glyph_atlas::GlyphAtlas,
    pub camera_viewports: std::collections::HashMap<nightshade::ecs::Entity, CameraViewport>,
    pub paint_target: Option<PaintTarget>,
    pub brdf_lut_texture: wgpu::Texture,
    pub brdf_lut_view: wgpu::TextureView,
    pub mip_generator: mip_generator::MipGenerator,
    pub shader_composer: ShaderComposer,
    pub timing: timing::GpuTiming,
    pub(crate) depth_pick: DepthPickState,
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) screenshot: ScreenshotState,
    pub render_buffer_size: (u32, u32),
    pub(crate) window_render_state: WindowRenderState,
    pub(crate) surface_skips: u32,
    pub frame_state: FrameState,
    pub(crate) texture_store: crate::renderer::wgpu::texture_cache::TextureStore,
    pub gpu_profile: crate::renderer::config::GpuProfile,
    pub bindless: crate::renderer::wgpu::caps::BindlessConfig,
}

pub struct RenderTargets {
    pub depth: rendergraph::ResourceId,
    pub scene_color: rendergraph::ResourceId,
    pub compute_output: rendergraph::ResourceId,
    pub aa_output: rendergraph::ResourceId,
    pub swapchain: rendergraph::ResourceId,
    pub viewport_resource: rendergraph::ResourceId,
    pub swapchain_depth: rendergraph::ResourceId,
    pub entity_id: rendergraph::ResourceId,
    pub view_normals: rendergraph::ResourceId,
    pub spotlight_shadow_atlas: rendergraph::ResourceId,
    pub glyph_atlas: rendergraph::ResourceId,
    pub brdf_lut: rendergraph::ResourceId,
    pub ibl_irradiance: rendergraph::ResourceId,
    pub ibl_prefiltered: rendergraph::ResourceId,
    pub ibl_irradiance_b: rendergraph::ResourceId,
    pub ibl_prefiltered_b: rendergraph::ResourceId,
    pub point_shadow_cubemap: rendergraph::ResourceId,
    pub bone_matrices: rendergraph::ResourceId,
    pub transmission_grab: rendergraph::ResourceId,
    pub shadow_depth: rendergraph::ResourceId,
    pub velocity: rendergraph::ResourceId,
    pub outline_mask: rendergraph::ResourceId,
    pub ssao_raw: rendergraph::ResourceId,
    pub ssao: rendergraph::ResourceId,
    pub ssgi_raw: rendergraph::ResourceId,
    pub ssgi: rendergraph::ResourceId,
    pub ssr_raw: rendergraph::ResourceId,
    pub ssr: rendergraph::ResourceId,
}

pub(crate) struct DepthPickState {
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
    pub camera: Option<nightshade::ecs::Entity>,
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) struct ScreenshotState {
    pub staging_buffer: wgpu::Buffer,
    pub pending: bool,
    pub map_complete: std::sync::Arc<std::sync::atomic::AtomicBool>,
    pub path: Option<std::path::PathBuf>,
    pub width: u32,
    pub height: u32,
    pub max_dimension: Option<u32>,
    pub source_is_bgra: bool,
    pub writer: Option<std::thread::JoinHandle<()>>,
}

#[cfg(not(target_arch = "wasm32"))]
impl Drop for ScreenshotState {
    fn drop(&mut self) {
        if let Some(writer) = self.writer.take() {
            writer.join().ok();
        }
    }
}

pub struct FrameState {
    pub index: u64,
    pub last_settings_signature: Option<u64>,
    pub captured_ibl_atmosphere: Option<crate::renderer::config::Atmosphere>,
    pub captured_ibl_hour: f32,
    pub captured_day_night_snapshots: bool,
    pub captured_ibl_bracket: Option<(usize, usize)>,
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
