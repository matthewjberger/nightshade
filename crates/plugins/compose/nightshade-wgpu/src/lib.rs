pub mod commands;
#[cfg(feature = "default-plugins")]
pub mod default_plugins;
mod plugin;
pub mod pre_render;
mod renderer;
pub mod resources;
pub mod systems;

pub use ::wgpu;
pub use commands::RenderAppExt;
#[cfg(feature = "default-plugins")]
pub use default_plugins::DefaultPlugins;
pub use plugin::RenderPlugin;
pub use systems::frame::WgpuFrameRenderer;

pub mod backend {
    pub use crate::renderer::config::EffectiveShading;
    pub use crate::renderer::rendergraph::{
        MaterialUpdate, PassDecl, PassExecutionContext, PassNode, PassPosition, PassStage,
        RenderGraph, RenderGraphError, ResourceId, ResourcePublisher, ResourceReader, Result,
        render_graph_add_color_texture, render_graph_add_pass_at, render_graph_bind_pass_slot,
        render_graph_get_pass_mut, render_graph_set_external_texture,
    };
    pub use crate::renderer::text_mesh::{
        TextMeshParams, generate_text_mesh_screen, generate_text_mesh_world,
    };
    pub use crate::renderer::texture_extent::texture_extent;
    pub use crate::renderer::wgpu::HDR_FORMAT;
    pub use crate::renderer::wgpu::WgpuRenderer;
    pub use crate::renderer::wgpu::build;
    pub use crate::renderer::wgpu::caps;
    pub use crate::renderer::wgpu::caps::BindlessConfig;
    pub use crate::renderer::wgpu::frame::FrameInputsFn;
    pub use crate::renderer::wgpu::frame::render_frame;
    pub use crate::renderer::wgpu::initialization::{DeviceConfig, negotiate_device_config};
    pub use crate::renderer::wgpu::install::{
        install_lit_pass, install_scene_pass, install_swapchain_pass,
    };
    pub use crate::renderer::wgpu::material_texture_arrays::MATERIAL_SAMPLER_COUNT;
    pub use crate::renderer::wgpu::material_texture_arrays::{
        MaterialTextureArrays, MaterialTextureLayer,
    };
    pub use crate::renderer::wgpu::pass_build::PassBuild;
    pub use crate::renderer::wgpu::pass_sync::publish_glyph_atlas;
    pub use crate::renderer::wgpu::pass_sync::{
        generate_screen_text_mesh, generate_world_text_mesh, glyph_atlas_revision,
        publish_glyph_atlas_if_grown,
    };
    pub use crate::renderer::wgpu::passes::grow_storage_buffer;
    pub use crate::renderer::wgpu::passes::{fullscreen_pipeline, sampled_texture_uniform_layout};
    pub use crate::renderer::wgpu::placeholder;
    pub use crate::renderer::wgpu::placeholder::{placeholder_depth_texture, placeholder_texture};
    pub use crate::renderer::wgpu::presentation::install_presentation_passes;
    pub use crate::renderer::wgpu::render_configs::{
        CameraFrameInputs, CameraProjectionParams, RenderInputs, SingleViewHost, ViewFrame,
        compose_single_view, restore_single_view,
    };
    pub use crate::renderer::wgpu::shader_compose::ShaderComposer;
    pub use crate::renderer::wgpu::stagger;
    pub use crate::renderer::wgpu::texture_array_pool;
    pub use crate::renderer::wgpu::texture_cache::{
        TextureStore, texture_store_contains, texture_store_get, texture_store_names,
    };
}
