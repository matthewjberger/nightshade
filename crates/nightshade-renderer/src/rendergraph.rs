//! Frame graph rendering system for composable render pipelines.
//!
//! Build complex rendering pipelines by declaring passes and their resource dependencies:
//!
//! - [`RenderGraph`]: Main graph structure that schedules and executes render passes
//! - [`PassNode`]: Trait for implementing custom render passes
//! - [`PassExecutionContext`]: Runtime context for accessing resources during execution
//! - [`ResourceId`]: Handle to a graph-managed texture or buffer
//!
//! The render graph automatically handles:
//! - Pass ordering based on resource dependencies
//! - Transient resource allocation and aliasing
//! - Load/store operation optimization
//! - Dead pass culling
//!
//! # Basic Graph Setup
//!
//! ```ignore
//! use nightshade_renderer::rendergraph::{RenderGraph, PassNode, PassExecutionContext};
//!
//! let mut graph = render_graph_new();
//!
//! // Declare resources
//! let depth = render_graph_add_depth_texture(&mut graph, "depth")
//!     .size(1920, 1080)
//!     .clear_depth(0.0)
//!     .transient();
//!
//! let scene_color = render_graph_add_color_texture(&mut graph, "scene_color")
//!     .format(wgpu::TextureFormat::Rgba16Float)
//!     .size(1920, 1080)
//!     .clear_color(wgpu::Color::BLACK)
//!     .transient();
//!
//! let swapchain = render_graph_add_color_texture(&mut graph, "swapchain")
//!     .format(wgpu::TextureFormat::Bgra8Unorm)
//!     .external();  // Provided each frame
//!
//! // Add passes with their resource bindings
//! render_graph_add_pass(&mut graph, Box::new(mesh_pass), &[
//!     ("color", scene_color),
//!     ("depth", depth),
//! ])?;
//!
//! render_graph_add_pass(&mut graph, Box::new(post_process_pass), &[
//!     ("input", scene_color),
//!     ("output", swapchain),
//! ])?;
//!
//! render_graph_compile(&mut graph)?;
//! ```
//!
//! # Implementing a Custom Pass
//!
//! ```ignore
//! struct MyPass {
//!     pipeline: wgpu::RenderPipeline,
//! }
//!
//! impl PassNode for MyPass {
//!     fn name(&self) -> &str { "my_pass" }
//!
//!     fn reads(&self) -> Vec<&str> { vec!["input"] }
//!     fn writes(&self) -> Vec<&str> { vec!["output"] }
//!
//!     fn execute<'r, 'e>(
//!         &mut self,
//!         ctx: PassExecutionContext<'r, 'e>,
//!     ) -> Result<Vec<SubGraphRunCommand<'r>>> {
//!         let (output_view, load_op, store_op) = ctx.get_color_attachment("output")?;
//!         let input_view = ctx.get_texture_view("input")?;
//!
//!         let mut pass = ctx.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
//!             color_attachments: &[Some(wgpu::RenderPassColorAttachment {
//!                 view: output_view,
//!                 resolve_target: None,
//!                 ops: wgpu::Operations { load: load_op, store: store_op },
//!             })],
//!             ..Default::default()
//!         });
//!
//!         pass.set_pipeline(&self.pipeline);
//!         // Bind input_view to a bind group...
//!         pass.draw(0..3, 0..1);
//!
//!         Ok(vec![])
//!     }
//! }
//! ```
//!
//! # Executing the Graph
//!
//! ```ignore
//! // Each frame:
//! render_graph_set_external_texture(&mut graph, swapchain_id, None, surface_view, width, height);
//!
//! let command_buffers = render_graph_execute(&mut graph, &device, &queue, &world)?;
//! queue.submit(command_buffers);
//! ```
//!
//! # Resource Types
//!
//! | Type | Description |
//! |------|-------------|
//! | Transient | Graph allocates/manages lifetime, can be aliased |
//! | External | Provided each frame (swapchain, imported textures) |
//!
//! # Pass Methods
//!
//! | Method | Description |
//! |--------|-------------|
//! | `name()` | Unique identifier for the pass |
//! | `reads()` | Slots this pass reads from |
//! | `writes()` | Slots this pass writes to |
//! | `reads_writes()` | Slots used for both read and write |
//! | `optional_reads()` | Reads tolerated to be unbound (e.g. `ssgi`, `ssr`) |
//! | `prepare()` | Called before execute, for uploading uniforms |
//! | `invalidate_bind_groups()` | Called when a touched resource is rebound; drop cached bind groups |
//! | `execute()` | Record GPU commands |
//! | `runs_in_full_phase()` | Run during a normal full render (default `true`) |
//! | `runs_in_compose_only_phase()` | Run during a cached-viewport compose-only frame (default `false`) |
//!
//! # Context Methods
//!
//! | Method | Description |
//! |--------|-------------|
//! | `get_texture_view(slot)` | Get texture view for reading |
//! | `get_color_attachment(slot)` | Get view + load/store ops for color output |
//! | `get_depth_attachment(slot)` | Get view + load/store ops for depth output |
//! | `get_buffer(slot)` | Get buffer for compute passes |
//! | `get_texture_size(slot)` | Get dimensions of a texture |
//!
//! # Toggling Passes
//!
//! Passes can be enabled or disabled by name on a built graph:
//!
//! ```ignore
//! render_graph_set_pass_enabled(&mut graph, "bloom_pass", false);
//! ```

mod error;
mod graph;
mod pass;
mod resources;

pub use error::{RenderGraphError, Result};
pub use graph::{
    PoolSlot, RenderGraph, ResourceAliasingInfo, render_graph_add_buffer,
    render_graph_add_color_texture, render_graph_add_dependency, render_graph_add_depth_texture,
    render_graph_add_pass, render_graph_add_sub_graph, render_graph_compile,
    render_graph_enable_only_passes, render_graph_execute, render_graph_execute_with_phase,
    render_graph_external_color, render_graph_get_pass_mut, render_graph_get_sub_graph,
    render_graph_get_sub_graph_mut, render_graph_get_texture, render_graph_get_texture_view,
    render_graph_new, render_graph_pass, render_graph_resize_all_transient_proportional,
    render_graph_resize_all_transient_textures, render_graph_resize_transient_resource,
    render_graph_resource_pool, render_graph_resources_mut,
    render_graph_restore_pass_enabled_states, render_graph_save_pass_enabled_states,
    render_graph_set_external_buffer, render_graph_set_external_texture,
    render_graph_set_pass_enabled, render_graph_transient_color_from_template,
    render_graph_transient_color_from_template_with_clear,
};
pub use pass::{
    BufferBuilder, ColorTextureBuilder, DepthTextureBuilder, ExecutePhase, GraphNode, PassBuilder,
    PassExecutionContext, PassNode, ResourcePool, ResourceTemplate, SlotValue, SubGraphInputSlot,
    SubGraphRunCommand,
};
pub use resources::{
    RenderGraphBufferDescriptor, RenderGraphResources, RenderGraphTextureDescriptor,
    ResourceDescriptor, ResourceHandle, ResourceId, ResourceType,
};
