pub mod commands;
#[cfg(feature = "default-plugins")]
pub mod default_plugins;
mod plugin;
pub mod pre_render;
mod renderer;
pub mod resources;
pub mod systems;

pub use ::ash;
pub use commands::VulkanAppExt;
#[cfg(feature = "default-plugins")]
pub use default_plugins::DefaultPlugins;
pub use plugin::RenderPlugin;
pub use systems::frame::VulkanFrameRenderer;

pub mod backend {
    pub use crate::renderer::context::{Context, FRAMES_IN_FLIGHT};
    pub use crate::renderer::memory::{
        Buffer, Image, create_buffer, destroy_buffer, upload_to_buffer, write_buffer,
    };
    pub use crate::renderer::pass::{
        Pass, PassRecord, PassSetup, PassStage, create_shader_module, read_spirv,
    };
    pub use crate::renderer::{
        VulkanRenderer, install_scene_pass, install_swapchain_pass, pass_mut, pass_setup,
        pass_with_context,
    };
    pub use gpu_allocator::MemoryLocation;
}
