//! Built-in render passes for the default pipeline.
//!
//! These passes implement [`PassNode`](super::rendergraph::PassNode) and can be used
//! directly or as references for custom pass implementations.
//!
//! # Geometry Passes
//!
//! | Pass | Description |
//! |------|-------------|
//! | [`MeshPass`] | Static mesh rendering with lighting |
//! | [`SkinnedMeshPass`] | Animated skeletal mesh rendering |
//! | [`DecalPass`] | Projected decal rendering |
//! | [`ParticlePass`] | GPU particle billboard rendering |
//! | [`ClothPass`] | GPU cloth simulation and rendering |
//! | [`TextPass`] | 3D text rendering |
//! | [`LinesPass`] | Debug line rendering |
//! | [`GridPass`] | Infinite ground grid |
//! | `GrassPass` | GPU-driven procedural grass (`grass` feature) |
//! | `TerrainPass` | Clipmap terrain with GPU height cache (`terrain` feature) |
//! | [`SceneOverlayPass`] | Grid, lines, particles, and text in one render pass |
//! | [`SkyPass`] | Procedural sky/atmosphere |
//! | [`UiPass`] | UI rectangle + text rendering (batched) |
//!
//! # Shadow Passes
//!
//! | Pass | Description |
//! |------|-------------|
//! | [`ShadowDepthPass`] | Cascaded shadow map generation |
//!
//! Shadow mapping uses 4 cascades (`NUM_SHADOW_CASCADES`) for directional lights.
//!
//! # Post-Processing Passes
//!
//! | Pass | Description |
//! |------|-------------|
//! | [`PostProcessPass`] | Final compositing with tonemapping |
//! | [`BloomPass`] | HDR bloom effect |
//! | [`SsaoPass`] | Screen-space ambient occlusion |
//! | [`SsaoBlurPass`] | SSAO blur for soft shadows |
//! | [`SsgiPass`] | Screen-space global illumination |
//! | [`SsgiBlurPass`] | SSGI bilateral blur |
//! | [`DepthOfFieldPass`] | Bokeh depth of field |
//! | `OutlinePass` | Selection outline rendering |
//! | `SelectionMaskPass` | Selection mask generation |
//! | [`BlitPass`] | Simple texture copy |
//!
//! # Using Built-in Passes
//!
//! ```ignore
//! fn configure_render_graph(
//!     &mut self,
//!     graph: &mut RenderGraph<World>,
//!     device: &wgpu::Device,
//!     surface_format: wgpu::TextureFormat,
//!     resources: RenderResources,
//! ) {
//!     // Add bloom post-processing
//!     let bloom = BloomPass::new(device, width, height);
//!     render_graph_pass(graph, Box::new(bloom))
//!         .slot("hdr", resources.scene_color)
//!         .slot("bloom", bloom_texture);
//!
//!     // Add SSAO
//!     let ssao = SsaoPass::new(device);
//!     render_graph_pass(graph, Box::new(ssao))
//!         .slot("depth", resources.depth)
//!         .slot("view_normals", resources.view_normals)
//!         .slot("ssao_raw", ssao_raw);
//!
//!     let ssao_blur = SsaoBlurPass::new(device);
//!     render_graph_pass(graph, Box::new(ssao_blur))
//!         .slot("ssao_raw", ssao_raw)
//!         .slot("depth", resources.depth)
//!         .slot("view_normals", resources.view_normals)
//!         .slot("ssao", ssao_final);
//!
//!     // Final post-process with tonemapping
//!     let postprocess = PostProcessPass::new(device, surface_format, bloom_intensity);
//!     render_graph_pass(graph, Box::new(postprocess))
//!         .slot("hdr", resources.scene_color)
//!         .slot("bloom", bloom_texture)
//!         .slot("ssao", ssao_final)
//!         .slot("output", resources.swapchain);
//! }
//! ```
//!
//! # Pass Configuration
//!
//! Most passes accept configuration in their constructors:
//!
//! ```ignore
//! // Bloom with custom mip levels
//! let bloom = BloomPass::new(device, width, height);
//!
//! // SSAO (tuned at runtime via RenderInputs::settings ssao_* fields)
//! let ssao = SsaoPass::new(device);
//!
//! // Post-process with bloom intensity
//! let postprocess = PostProcessPass::new(device, format, 0.3);  // 30% bloom
//! ```

/// Passes that run before the scene draws, such as pick keepalive.
pub mod setup;
pub use setup::PickKeepalivePass;

pub mod geometry;
#[cfg(feature = "grass")]
pub use geometry::GrassPass;
#[cfg(feature = "meshlet")]
pub use geometry::MeshletPass;
#[cfg(feature = "terrain")]
pub use geometry::TerrainPass;
pub use geometry::{
    ClothPass, DecalPass, GridPass, LinesPass, MeshPass, ParticlePass, SceneOverlayPass,
    SkinnedMeshPass, SkyPass, TextPass, UiImagePass, UiPass, WaterPass,
};

pub mod shadow_depth;
pub use shadow_depth::{
    CASCADE_ATLAS_SIZE, CASCADE_SLOT_RESOLUTION, NUM_SHADOW_CASCADES, ShadowDepthPass,
    SpotlightShadowData, SpotlightShadowSlot,
};

/// Screen-space post-processing passes: bloom, SSAO, SSGI, SSR, depth of
/// field, TAA, color grading, blit, and viewport compose.
pub mod postprocess;
pub use postprocess::{
    BlitPass, BloomPass, DepthOfFieldPass, EffectsPass, PostProcessPass, SsaoBlurPass, SsaoPass,
    SsgiBlurPass, SsgiPass, SsrBlurPass, SsrPass, TaaPass, ViewportComposePass,
};
#[cfg(feature = "debug_render")]
pub use postprocess::{OutlinePass, SelectionMaskPass};

#[cfg(feature = "egui")]
mod egui_pass;
#[cfg(feature = "egui")]
pub use egui_pass::EguiPass;

/// Grows a storage buffer to hold at least `required` elements of `T`, doubling
/// the capacity so reallocation amortizes. Returns `true` when the buffer was
/// recreated, so the caller can rebuild any bind groups that referenced the old
/// buffer. No-op (returns `false`) while the current capacity already fits.
pub fn grow_storage_buffer<T>(
    device: &wgpu::Device,
    buffer: &mut wgpu::Buffer,
    capacity: &mut usize,
    required: usize,
    usage: wgpu::BufferUsages,
    label: &str,
) -> bool {
    if required <= *capacity {
        return false;
    }
    let new_capacity = (required as f32 * 2.0).ceil() as usize;
    *buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: (std::mem::size_of::<T>() * new_capacity) as u64,
        usage,
        mapped_at_creation: false,
    });
    *capacity = new_capacity;
    true
}
