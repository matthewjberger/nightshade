//! Render pass trait and builder types for the render graph.
//!
//! This module provides the core abstractions for implementing custom render passes:
//!
//! - [`PassNode`]: Trait that all render passes must implement
//! - [`PassExecutionContext`]: Runtime context passed to `execute()` with resource access
//! - [`ColorTextureBuilder`], [`DepthTextureBuilder`], [`BufferBuilder`]: Fluent builders for resources
//!
//! # Implementing PassNode
//!
//! Every render pass implements [`PassNode`] to declare its resource dependencies and
//! execute GPU commands:
//!
//! ```ignore
//! struct BlurPass {
//!     pipeline: wgpu::RenderPipeline,
//!     bind_group_layout: wgpu::BindGroupLayout,
//!     bind_group: Option<wgpu::BindGroup>,
//!     sampler: wgpu::Sampler,
//! }
//!
//! impl PassNode<MyInputs> for BlurPass {
//!     fn name(&self) -> &str { "blur_pass" }
//!
//!     // Declare which slots this pass reads from
//!     fn reads(&self) -> Vec<&str> { vec!["input"] }
//!
//!     // Declare which slots this pass writes to
//!     fn writes(&self) -> Vec<&str> { vec!["output"] }
//!
//!     // Optional: prepare bind groups before execute
//!     fn prepare(&mut self, device: &Device, _queue: &wgpu::Queue, _inputs: &MyInputs) {
//!         // Bind groups are recreated when resources change
//!     }
//!
//!     // Optional: called when bound resources change
//!     fn invalidate_bind_groups(&mut self) {
//!         self.bind_group = None;
//!     }
//!
//!     fn execute<'r, 'e>(
//!         &mut self,
//!         ctx: PassExecutionContext<'r, 'e, MyInputs>,
//!     ) -> Result<Vec<SubGraphRunCommand<'r>>> {
//!         // Skip if pass is disabled
//!         if !ctx.is_pass_enabled() {
//!             return Ok(vec![]);
//!         }
//!
//!         // Get resources through slots
//!         let input_view = ctx.get_texture_view("input")?;
//!         let (output_view, load_op, store_op) = ctx.get_color_attachment("output")?;
//!
//!         // Create bind group if needed
//!         if self.bind_group.is_none() {
//!             self.bind_group = Some(ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
//!                 layout: &self.bind_group_layout,
//!                 entries: &[
//!                     wgpu::BindGroupEntry {
//!                         binding: 0,
//!                         resource: wgpu::BindingResource::TextureView(input_view),
//!                     },
//!                     wgpu::BindGroupEntry {
//!                         binding: 1,
//!                         resource: wgpu::BindingResource::Sampler(&self.sampler),
//!                     },
//!                 ],
//!                 label: Some("blur_bind_group"),
//!             }));
//!         }
//!
//!         // Record render commands
//!         let mut pass = ctx.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
//!             label: Some("blur_pass"),
//!             color_attachments: &[Some(wgpu::RenderPassColorAttachment {
//!                 view: output_view,
//!                 resolve_target: None,
//!                 ops: wgpu::Operations { load: load_op, store: store_op },
//!             })],
//!             depth_stencil_attachment: None,
//!             ..Default::default()
//!         });
//!
//!         pass.set_pipeline(&self.pipeline);
//!         pass.set_bind_group(0, self.bind_group.as_ref().unwrap(), &[]);
//!         pass.draw(0..3, 0..1);  // Fullscreen triangle
//!
//!         Ok(vec![])
//!     }
//! }
//! ```
//!
//! # PassExecutionContext
//!
//! The context provides access to graph resources during execution:
//!
//! ```ignore
//! // Get a texture view for sampling
//! let view = ctx.get_texture_view("my_input")?;
//!
//! // Get attachment with automatic load/store ops
//! let (view, load, store) = ctx.get_color_attachment("color_output")?;
//! let (depth_view, depth_load, depth_store) = ctx.get_depth_attachment("depth")?;
//!
//! // Get texture dimensions
//! let (width, height) = ctx.get_texture_size("output")?;
//!
//! // Get a buffer (for compute passes)
//! let buffer = ctx.get_buffer("data_buffer")?;
//!
//! // Access the device for creating bind groups
//! let bind_group = ctx.device.create_bind_group(...);
//!
//! // Access the command encoder
//! let render_pass = ctx.encoder.begin_render_pass(...);
//! ```
//!
//! # Resource Builders
//!
//! Create graph resources with fluent builders:
//!
//! ```ignore
//! // Color texture (transient - graph manages lifetime)
//! let hdr = render_graph_add_color_texture(&mut graph, "hdr")
//!     .format(wgpu::TextureFormat::Rgba16Float)
//!     .size(1920, 1080)
//!     .clear_color(wgpu::Color::BLACK)
//!     .transient();
//!
//! // Color texture (external - you provide each frame)
//! let swapchain = render_graph_add_color_texture(&mut graph, "swapchain")
//!     .format(wgpu::TextureFormat::Bgra8UnormSrgb)
//!     .external();
//!
//! // Depth texture
//! let depth = render_graph_add_depth_texture(&mut graph, "depth")
//!     .size(1920, 1080)
//!     .format(wgpu::TextureFormat::Depth32Float)
//!     .clear_depth(0.0)
//!     .transient();
//!
//! // Buffer
//! let buffer = render_graph_add_buffer(&mut graph, "compute_data")
//!     .size(1024 * 1024)
//!     .usage(wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST)
//!     .transient();
//! ```

use super::error::{RenderGraphError, Result};
use super::graph::{render_graph_external_color, render_graph_transient_color_from_template};
use super::resources::{
    RenderGraphBufferDescriptor, RenderGraphResources, RenderGraphTextureDescriptor,
    ResourceHandle, ResourceId, ResourceType,
};
use std::collections::HashMap;
use wgpu::{
    Buffer, BufferUsages, CommandEncoder, Device, TextureFormat, TextureUsages, TextureView,
};

/// Deferred request to run a named sub-graph with a set of input slot values, produced by a pass's `execute`.
pub struct SubGraphRunCommand<'a> {
    /// Name of the sub-graph to run.
    pub sub_graph_name: String,
    /// Slot values bound as the sub-graph's inputs.
    pub inputs: Vec<SlotValue<'a>>,
}

/// Runtime context handed to [`PassNode::execute`] for accessing graph resources and recording commands.
pub struct PassExecutionContext<'r, 'e, C = ()> {
    /// Command encoder the pass records into.
    pub encoder: &'e mut CommandEncoder,
    /// Resolved graph resources for the current frame.
    pub resources: &'r RenderGraphResources,
    /// The `wgpu` device.
    pub device: &'r Device,
    /// The `wgpu` queue.
    pub queue: &'r wgpu::Queue,
    pub(super) slot_mappings: &'r HashMap<String, ResourceId>,
    /// Per-frame configuration threaded through the graph.
    pub configs: &'r C,
    pub(crate) sub_graph_commands: Vec<SubGraphRunCommand<'r>>,
    pub(super) node_index: petgraph::graph::NodeIndex,
    pub(super) clear_ops: &'r std::collections::HashSet<(petgraph::graph::NodeIndex, ResourceId)>,
    pub(super) store_ops: &'r HashMap<(petgraph::graph::NodeIndex, ResourceId), wgpu::StoreOp>,
    pub(super) pass_enabled: bool,
}

impl<'r, 'e, C> PassExecutionContext<'r, 'e, C> {
    /// Returns whether this pass is enabled for the current frame.
    pub fn is_pass_enabled(&self) -> bool {
        self.pass_enabled
    }

    /// Resolves a slot name to its bound [`ResourceId`].
    pub fn get_slot(&self, slot: &str) -> Result<ResourceId> {
        self.slot_mappings
            .get(slot)
            .copied()
            .ok_or_else(|| RenderGraphError::SlotNotFound {
                slot: slot.to_string(),
                pass: "unknown".to_string(),
            })
    }

    /// Returns the texture view bound to `slot`.
    pub fn get_texture_view(&self, slot: &str) -> Result<&'r wgpu::TextureView> {
        let resource_id = self.get_slot(slot)?;
        self.resources.get_texture_view(resource_id).ok_or_else(|| {
            RenderGraphError::ResourceNotBound {
                resource: slot.to_string(),
                id: resource_id,
            }
        })
    }

    /// Returns the texture bound to `slot`.
    pub fn get_texture(&self, slot: &str) -> Result<&'r wgpu::Texture> {
        let resource_id = self.get_slot(slot)?;
        self.resources
            .get_texture(resource_id)
            .ok_or_else(|| RenderGraphError::ResourceNotBound {
                resource: slot.to_string(),
                id: resource_id,
            })
    }

    /// Returns the color attachment view for `slot` with its resolved load and store ops.
    pub fn get_color_attachment(
        &self,
        slot: &str,
    ) -> Result<(
        &'r wgpu::TextureView,
        wgpu::LoadOp<wgpu::Color>,
        wgpu::StoreOp,
    )> {
        let resource_id = self.get_slot(slot)?;
        self.resources.get_color_attachment(
            resource_id,
            self.node_index,
            self.clear_ops,
            self.store_ops,
        )
    }

    /// Returns the depth attachment view for `slot` with its resolved load and store ops.
    pub fn get_depth_attachment(
        &self,
        slot: &str,
    ) -> Result<(&'r wgpu::TextureView, wgpu::LoadOp<f32>, wgpu::StoreOp)> {
        let resource_id = self.get_slot(slot)?;
        self.resources.get_depth_attachment(
            resource_id,
            self.node_index,
            self.clear_ops,
            self.store_ops,
        )
    }

    /// Returns the buffer bound to `slot`.
    pub fn get_buffer(&self, slot: &str) -> Result<&'r wgpu::Buffer> {
        let resource_id = self.get_slot(slot)?;
        let handle = self.resources.get_handle(resource_id).ok_or_else(|| {
            RenderGraphError::ResourceNotBound {
                resource: slot.to_string(),
                id: resource_id,
            }
        })?;

        match handle {
            ResourceHandle::ExternalBuffer { buffer }
            | ResourceHandle::TransientBuffer { buffer } => Ok(buffer),
            _ => Err(RenderGraphError::TypeMismatch {
                operation: "get_buffer".to_string(),
                actual_type: "texture".to_string(),
                resource: slot.to_string(),
            }),
        }
    }

    /// Returns the `(width, height)` in pixels of the texture bound to `slot`.
    pub fn get_texture_size(&self, slot: &str) -> Result<(u32, u32)> {
        let resource_id = self.get_slot(slot)?;
        let descriptor = self.resources.get_descriptor(resource_id).ok_or_else(|| {
            RenderGraphError::DescriptorNotFound {
                resource: slot.to_string(),
                id: resource_id,
            }
        })?;

        match &descriptor.resource_type {
            ResourceType::TransientColor {
                descriptor: texture_desc,
                ..
            }
            | ResourceType::TransientDepth {
                descriptor: texture_desc,
                ..
            } => Ok((texture_desc.width, texture_desc.height)),
            ResourceType::ExternalColor { .. } | ResourceType::ExternalDepth { .. } => {
                let handle = self.resources.get_handle(resource_id).ok_or_else(|| {
                    RenderGraphError::ResourceNotBound {
                        resource: slot.to_string(),
                        id: resource_id,
                    }
                })?;

                match handle {
                    ResourceHandle::ExternalTexture { width, height, .. } => Ok((*width, *height)),
                    _ => Err(RenderGraphError::TypeMismatch {
                        operation: "get_texture_size".to_string(),
                        actual_type: "transient_texture".to_string(),
                        resource: slot.to_string(),
                    }),
                }
            }
            _ => Err(RenderGraphError::TypeMismatch {
                operation: "get_texture_size".to_string(),
                actual_type: "buffer".to_string(),
                resource: slot.to_string(),
            }),
        }
    }

    /// Queues a sub-graph to run after this pass with the given input slot values.
    pub fn run_sub_graph(&mut self, sub_graph_name: String, inputs: Vec<SlotValue<'r>>) {
        self.sub_graph_commands.push(SubGraphRunCommand {
            sub_graph_name,
            inputs,
        });
    }

    /// Consumes the context and returns the queued sub-graph run commands.
    pub fn into_sub_graph_commands(self) -> Vec<SubGraphRunCommand<'r>> {
        self.sub_graph_commands
    }
}

/// Phase for `render_graph_execute_with_phase`. `Full` runs every
/// enabled pass; `ComposeOnly` runs only passes that override
/// [`PassNode::runs_in_compose_only_phase`] to return `true`. The
/// renderer uses `ComposeOnly` for camera tiles whose update mode says
/// they should reuse their cached viewport texture rather than
/// re-render this frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ExecutePhase {
    /// Runs every enabled pass.
    #[default]
    Full,
    /// Runs only passes that opt into [`PassNode::runs_in_compose_only_phase`].
    ComposeOnly,
}

/// A node in the render graph that declares its resource dependencies and records GPU commands.
#[cfg(not(target_arch = "wasm32"))]
pub trait PassNode<C = ()>: Send + Sync + std::any::Any {
    /// Unique name of the pass within the graph.
    fn name(&self) -> &str;
    /// Slot names this pass reads from.
    fn reads(&self) -> Vec<&str>;
    /// Slot names this pass writes to.
    fn writes(&self) -> Vec<&str>;
    /// Slot names this pass both reads and writes.
    fn reads_writes(&self) -> Vec<&str> {
        Vec::new()
    }
    /// Slot names this pass reads when present, tolerating absence.
    fn optional_reads(&self) -> Vec<&str> {
        Vec::new()
    }
    /// Prepares GPU state each frame before [`PassNode::execute`].
    fn prepare(&mut self, _device: &Device, _queue: &wgpu::Queue, _configs: &C) {}
    /// Drops cached bind groups so they rebuild when bound resources change.
    fn invalidate_bind_groups(&mut self) {}
    /// Records the pass's GPU commands and returns any sub-graph run requests.
    fn execute<'r, 'e>(
        &mut self,
        context: PassExecutionContext<'r, 'e, C>,
    ) -> Result<Vec<SubGraphRunCommand<'r>>>;
    /// Whether this pass runs during the full per-camera render phase.
    /// Default `true`. Set to `false` for passes that should only run
    /// when the camera is reusing a cached viewport texture (e.g., a
    /// "preload cached camera tex into the post-process output"
    /// blit).
    fn runs_in_full_phase(&self) -> bool {
        true
    }
    /// Whether this pass runs during [`ExecutePhase::ComposeOnly`].
    /// Default `false`. Override on passes that present a cached
    /// viewport texture or editor UI overlay so they continue to draw
    /// even when the camera is reusing its previous frame's texture.
    fn runs_in_compose_only_phase(&self) -> bool {
        false
    }
}

/// A node in the render graph that declares its resource dependencies and records GPU commands.
#[cfg(target_arch = "wasm32")]
pub trait PassNode<C = ()>: std::any::Any {
    /// Unique name of the pass within the graph.
    fn name(&self) -> &str;
    /// Slot names this pass reads from.
    fn reads(&self) -> Vec<&str>;
    /// Slot names this pass writes to.
    fn writes(&self) -> Vec<&str>;
    /// Slot names this pass both reads and writes.
    fn reads_writes(&self) -> Vec<&str> {
        Vec::new()
    }
    /// Slot names this pass reads when present, tolerating absence.
    fn optional_reads(&self) -> Vec<&str> {
        Vec::new()
    }
    /// Prepares GPU state each frame before [`PassNode::execute`].
    fn prepare(&mut self, _device: &Device, _queue: &wgpu::Queue, _configs: &C) {}
    /// Drops cached bind groups so they rebuild when bound resources change.
    fn invalidate_bind_groups(&mut self) {}
    /// Records the pass's GPU commands and returns any sub-graph run requests.
    fn execute<'r, 'e>(
        &mut self,
        context: PassExecutionContext<'r, 'e, C>,
    ) -> Result<Vec<SubGraphRunCommand<'r>>>;
    /// Whether this pass runs during the full per-camera render phase.
    fn runs_in_full_phase(&self) -> bool {
        true
    }
    /// Whether this pass runs during [`ExecutePhase::ComposeOnly`].
    fn runs_in_compose_only_phase(&self) -> bool {
        false
    }
}

/// Graph node holding a pass together with its resolved resource bindings and enabled state.
pub struct GraphNode<C> {
    /// Pass name.
    pub name: String,
    /// Resource ids the pass reads.
    pub reads: Vec<ResourceId>,
    /// Resource ids the pass writes.
    pub writes: Vec<ResourceId>,
    /// Resource ids the pass both reads and writes.
    pub reads_writes: Vec<ResourceId>,
    /// Resource ids the pass reads when present.
    pub optional_reads: Vec<ResourceId>,
    /// The boxed pass implementation.
    pub pass: Box<dyn PassNode<C> + 'static>,
    /// Whether the pass runs this frame.
    pub enabled: bool,
}

/// A resource value bound to a sub-graph input slot.
pub enum SlotValue<'a> {
    /// A texture view with its dimensions and optional source texture.
    TextureView {
        /// Source texture, when available.
        texture: Option<&'a wgpu::Texture>,
        /// The bound texture view.
        view: &'a TextureView,
        /// Width in pixels.
        width: u32,
        /// Height in pixels.
        height: u32,
    },
    /// A bound buffer.
    Buffer(&'a Buffer),
}

/// Named input slot declared by a sub-graph.
#[derive(Clone)]
pub struct SubGraphInputSlot {
    /// Slot name.
    pub name: String,
}

/// Fluent builder for a graph color texture resource.
pub struct ColorTextureBuilder<'a, C = ()> {
    pub(super) graph: &'a mut super::graph::RenderGraph<C>,
    pub(super) name: String,
    pub(super) descriptor: RenderGraphTextureDescriptor,
    pub(super) clear_color: Option<wgpu::Color>,
    pub(super) force_store: bool,
    pub(super) fixed_size: bool,
}

impl<'a, C> ColorTextureBuilder<'a, C> {
    /// Sets the texture format.
    pub fn format(mut self, format: TextureFormat) -> Self {
        self.descriptor.format = format;
        self
    }

    /// Sets the texture dimensions in pixels.
    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.descriptor.width = width;
        self.descriptor.height = height;
        self
    }

    /// Sets the texture usage flags.
    pub fn usage(mut self, usage: TextureUsages) -> Self {
        self.descriptor.usage = usage;
        self
    }

    /// Sets the MSAA sample count.
    pub fn sample_count(mut self, count: u32) -> Self {
        self.descriptor.sample_count = count;
        self
    }

    /// Sets the mip level count.
    pub fn mip_levels(mut self, levels: u32) -> Self {
        self.descriptor.mip_level_count = levels;
        self
    }

    /// Sets the clear color applied on load.
    pub fn clear_color(mut self, color: wgpu::Color) -> Self {
        self.clear_color = Some(color);
        self
    }

    /// Marks the texture's contents as not requiring store after the pass.
    pub fn no_store(mut self) -> Self {
        self.force_store = false;
        self
    }

    /// Forces the texture's contents to be stored after the pass.
    pub fn force_store(mut self) -> Self {
        self.force_store = true;
        self
    }

    /// Keeps the texture at its declared size instead of tracking the render target.
    pub fn fixed_size(mut self) -> Self {
        self.fixed_size = true;
        self
    }

    /// Registers the resource as external, supplied each frame by the caller, and returns its id.
    pub fn external(self) -> ResourceId {
        self.graph.resources.register_external_resource(
            self.name,
            ResourceType::ExternalColor {
                clear_color: self.clear_color,
            },
        )
    }

    /// Registers the resource as transient, owned by the graph, and returns its id.
    pub fn transient(self) -> ResourceId {
        self.graph.resources.register_transient_resource_opts(
            self.name,
            ResourceType::TransientColor {
                descriptor: self.descriptor,
                clear_color: self.clear_color,
                force_store: self.force_store,
            },
            self.fixed_size,
        )
    }
}

/// Fluent builder for a graph depth texture resource.
pub struct DepthTextureBuilder<'a, C = ()> {
    pub(super) graph: &'a mut super::graph::RenderGraph<C>,
    pub(super) name: String,
    pub(super) descriptor: RenderGraphTextureDescriptor,
    pub(super) clear_depth: Option<f32>,
    pub(super) force_store: bool,
    pub(super) fixed_size: bool,
}

impl<'a, C> DepthTextureBuilder<'a, C> {
    /// Sets the texture format.
    pub fn format(mut self, format: TextureFormat) -> Self {
        self.descriptor.format = format;
        self
    }

    /// Sets the texture dimensions in pixels.
    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.descriptor.width = width;
        self.descriptor.height = height;
        self
    }

    /// Sets the texture usage flags.
    pub fn usage(mut self, usage: TextureUsages) -> Self {
        self.descriptor.usage = usage;
        self
    }

    /// Sets the MSAA sample count.
    pub fn sample_count(mut self, count: u32) -> Self {
        self.descriptor.sample_count = count;
        self
    }

    /// Sets the mip level count.
    pub fn mip_levels(mut self, levels: u32) -> Self {
        self.descriptor.mip_level_count = levels;
        self
    }

    /// Sets the number of array layers.
    pub fn array_layers(mut self, layers: u32) -> Self {
        self.descriptor.depth_or_array_layers = layers;
        self
    }

    /// Sets the depth clear value applied on load.
    pub fn clear_depth(mut self, depth: f32) -> Self {
        self.clear_depth = Some(depth);
        self
    }

    /// Marks the texture's contents as not requiring store after the pass.
    pub fn no_store(mut self) -> Self {
        self.force_store = false;
        self
    }

    /// Forces the texture's contents to be stored after the pass.
    pub fn force_store(mut self) -> Self {
        self.force_store = true;
        self
    }

    /// Keeps the texture at its declared size instead of tracking the render target.
    pub fn fixed_size(mut self) -> Self {
        self.fixed_size = true;
        self
    }

    /// Registers the resource as external, supplied each frame by the caller, and returns its id.
    pub fn external(self) -> ResourceId {
        self.graph.resources.register_external_resource(
            self.name,
            ResourceType::ExternalDepth {
                clear_depth: self.clear_depth,
            },
        )
    }

    /// Registers the resource as transient, owned by the graph, and returns its id.
    pub fn transient(self) -> ResourceId {
        self.graph.resources.register_transient_resource_opts(
            self.name,
            ResourceType::TransientDepth {
                descriptor: self.descriptor,
                clear_depth: self.clear_depth,
                force_store: self.force_store,
            },
            self.fixed_size,
        )
    }
}

/// Fluent builder for a graph buffer resource.
pub struct BufferBuilder<'a, C = ()> {
    pub(super) graph: &'a mut super::graph::RenderGraph<C>,
    pub(super) name: String,
    pub(super) descriptor: RenderGraphBufferDescriptor,
}

impl<'a, C> BufferBuilder<'a, C> {
    /// Sets the buffer size in bytes.
    pub fn size(mut self, size: u64) -> Self {
        self.descriptor.size = size;
        self
    }

    /// Sets the buffer usage flags.
    pub fn usage(mut self, usage: BufferUsages) -> Self {
        self.descriptor.usage = usage;
        self
    }

    /// Sets whether the buffer is mapped at creation.
    pub fn mapped_at_creation(mut self, mapped: bool) -> Self {
        self.descriptor.mapped_at_creation = mapped;
        self
    }

    /// Registers the buffer as external and returns its id.
    pub fn external(self) -> ResourceId {
        self.graph
            .resources
            .register_external_resource(self.name, ResourceType::ExternalBuffer)
    }

    /// Registers the buffer as transient and returns its id.
    pub fn transient(self) -> ResourceId {
        self.graph.resources.register_transient_resource(
            self.name,
            ResourceType::TransientBuffer {
                descriptor: self.descriptor,
            },
        )
    }
}

/// Reusable texture description for creating pooled graph resources.
#[derive(Clone)]
pub struct ResourceTemplate {
    pub(super) format: TextureFormat,
    pub(super) width: u32,
    pub(super) height: u32,
    pub(super) usage: TextureUsages,
    pub(super) sample_count: u32,
    pub(super) mip_level_count: u32,
    pub(super) dimension: wgpu::TextureDimension,
    pub(super) depth_or_array_layers: u32,
}

impl ResourceTemplate {
    /// Creates a template for a 2D texture with the given format and dimensions.
    pub fn new(format: TextureFormat, width: u32, height: u32) -> Self {
        Self {
            format,
            width,
            height,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            sample_count: 1,
            mip_level_count: 1,
            dimension: wgpu::TextureDimension::D2,
            depth_or_array_layers: 1,
        }
    }

    /// Sets the texture usage flags.
    pub fn usage(mut self, usage: TextureUsages) -> Self {
        self.usage = usage;
        self
    }

    /// Sets the MSAA sample count.
    pub fn sample_count(mut self, count: u32) -> Self {
        self.sample_count = count;
        self
    }

    /// Sets the mip level count.
    pub fn mip_levels(mut self, levels: u32) -> Self {
        self.mip_level_count = levels;
        self
    }

    /// Configures the template as a six-layer cube map.
    pub fn cube_map(mut self) -> Self {
        self.dimension = wgpu::TextureDimension::D2;
        self.depth_or_array_layers = 6;
        self
    }

    /// Sets the number of array layers.
    pub fn array_layers(mut self, layers: u32) -> Self {
        self.depth_or_array_layers = layers;
        self
    }

    /// Configures the template as a 3D texture with the given depth.
    pub fn dimension_3d(mut self, depth: u32) -> Self {
        self.dimension = wgpu::TextureDimension::D3;
        self.depth_or_array_layers = depth;
        self
    }
}

/// Fluent builder for adding a pass and its slot bindings to the graph.
pub struct PassBuilder<'a, C: 'static = ()> {
    pub(super) graph: &'a mut super::graph::RenderGraph<C>,
    pub(super) pass: Option<Box<dyn PassNode<C>>>,
    pub(super) slots: Vec<(&'static str, ResourceId)>,
}

impl<'a, C: 'static> PassBuilder<'a, C> {
    /// Binds a read slot to a resource.
    pub fn read(mut self, slot: &'static str, resource: ResourceId) -> Self {
        self.slots.push((slot, resource));
        self
    }

    /// Binds a write slot to a resource.
    pub fn write(mut self, slot: &'static str, resource: ResourceId) -> Self {
        self.slots.push((slot, resource));
        self
    }

    /// Binds a slot to a resource.
    pub fn slot(mut self, slot: &'static str, resource: ResourceId) -> Self {
        self.slots.push((slot, resource));
        self
    }

    /// Adds the configured pass to the graph and returns its node index.
    pub fn add(mut self) -> super::error::Result<petgraph::graph::NodeIndex> {
        let pass = self.pass.take().expect("PassBuilder consumed twice");
        let slots: Vec<(&str, ResourceId)> = self
            .slots
            .iter()
            .map(|(slot, resource)| (*slot, *resource))
            .collect();
        super::graph::render_graph_add_pass(self.graph, pass, &slots)
    }
}

/// Creates graph resources from a shared [`ResourceTemplate`].
pub struct ResourcePool<'a, C = ()> {
    pub(super) graph: &'a mut super::graph::RenderGraph<C>,
    pub(super) template: ResourceTemplate,
}

impl<'a, C: 'static> ResourcePool<'a, C> {
    /// Creates a transient color resource named `name` from the template.
    pub fn transient(&mut self, name: &str) -> ResourceId {
        render_graph_transient_color_from_template(self.graph, name, &self.template)
    }

    /// Creates one transient color resource per name from the template.
    pub fn transient_many(&mut self, names: &[&str]) -> Vec<ResourceId> {
        names.iter().map(|name| self.transient(name)).collect()
    }

    /// Registers an external color resource named `name`.
    pub fn external(&mut self, name: &str) -> ResourceId {
        render_graph_external_color(self.graph, name)
    }
}
