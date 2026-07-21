//! A wgpu renderer with a dependency-driven frame graph, built on
//! [`nightshade_ecs`].
//!
//! It provides:
//!
//! - **Render Graph**: Declarative pass-based rendering with automatic resource management
//! - **PBR Materials**: Physically-based rendering with metallic-roughness workflow
//! - **Shadow Mapping**: Cascaded shadow maps for directional lights
//! - **Post-Processing**: Bloom, SSAO, depth of field, tonemapping, color grading
//! - **GPU Particles**: Compute shader particle simulation and billboard rendering
//! - **Text Rendering**: Signed distance field text with GPU atlasing
//!
//! Custom passes implement [`PassNode`](wgpu::rendergraph::PassNode) over
//! [`RenderInputs`](wgpu::render_configs::RenderInputs) and are added to the
//! [`RenderGraph`](wgpu::rendergraph::RenderGraph) alongside the built-in ones.
//!
//! # Custom Shader Authoring
//!
//! Create custom render passes with WGSL shaders:
//!
//! ## 1. Write the Shader (WGSL)
//!
//! ```wgsl
//! // my_effect.wgsl
//! struct Uniforms {
//!     time: f32,
//!     intensity: f32,
//! }
//!
//! @group(0) @binding(0) var input_texture: texture_2d<f32>;
//! @group(0) @binding(1) var input_sampler: sampler;
//! @group(0) @binding(2) var<uniform> uniforms: Uniforms;
//!
//! struct VertexOutput {
//!     @builtin(position) position: vec4<f32>,
//!     @location(0) uv: vec2<f32>,
//! }
//!
//! @vertex
//! fn vs_main(@builtin(vertex_index) idx: u32) -> VertexOutput {
//!     // Fullscreen triangle (no vertex buffer needed)
//!     var positions = array<vec2<f32>, 3>(
//!         vec2(-1.0, -1.0),
//!         vec2(3.0, -1.0),
//!         vec2(-1.0, 3.0)
//!     );
//!     var uvs = array<vec2<f32>, 3>(
//!         vec2(0.0, 1.0),
//!         vec2(2.0, 1.0),
//!         vec2(0.0, -1.0)
//!     );
//!     var out: VertexOutput;
//!     out.position = vec4(positions[idx], 0.0, 1.0);
//!     out.uv = uvs[idx];
//!     return out;
//! }
//!
//! @fragment
//! fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
//!     let color = textureSample(input_texture, input_sampler, in.uv);
//!     let wave = sin(in.uv.x * 20.0 + uniforms.time) * uniforms.intensity;
//!     return color + vec4(wave, wave, wave, 0.0);
//! }
//! ```
//!
//! ## 2. Create the Pass Struct
//!
//! ```ignore
//! use nightshade::render::wgpu::rendergraph::{PassNode, PassExecutionContext, Result};
//!
//! pub struct MyEffectPass {
//!     pipeline: wgpu::RenderPipeline,
//!     bind_group_layout: wgpu::BindGroupLayout,
//!     bind_group: Option<wgpu::BindGroup>,
//!     uniform_buffer: wgpu::Buffer,
//!     sampler: wgpu::Sampler,
//!     time: f32,
//!     intensity: f32,
//! }
//!
//! impl MyEffectPass {
//!     pub fn new(device: &wgpu::Device, output_format: wgpu::TextureFormat) -> Self {
//!         // Load shader through naga_oil's composer for #ifdef and #import support
//!         let shader = nightshade::render::wgpu::shader_compose::compile_wgsl(
//!             device,
//!             "my_effect_shader",
//!             include_str!("my_effect.wgsl"),
//!         );
//!
//!         let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
//!             label: Some("my_effect_bgl"),
//!             entries: &[
//!                 wgpu::BindGroupLayoutEntry {
//!                     binding: 0,
//!                     visibility: wgpu::ShaderStages::FRAGMENT,
//!                     ty: wgpu::BindingType::Texture {
//!                         sample_type: wgpu::TextureSampleType::Float { filterable: true },
//!                         view_dimension: wgpu::TextureViewDimension::D2,
//!                         multisampled: false,
//!                     },
//!                     count: None,
//!                 },
//!                 wgpu::BindGroupLayoutEntry {
//!                     binding: 1,
//!                     visibility: wgpu::ShaderStages::FRAGMENT,
//!                     ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
//!                     count: None,
//!                 },
//!                 wgpu::BindGroupLayoutEntry {
//!                     binding: 2,
//!                     visibility: wgpu::ShaderStages::FRAGMENT,
//!                     ty: wgpu::BindingType::Buffer {
//!                         ty: wgpu::BufferBindingType::Uniform,
//!                         has_dynamic_offset: false,
//!                         min_binding_size: None,
//!                     },
//!                     count: None,
//!                 },
//!             ],
//!         });
//!
//!         let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
//!             label: Some("my_effect_layout"),
//!             bind_group_layouts: &[Some(&bind_group_layout)],
//!             immediate_size: 0,
//!         });
//!
//!         let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
//!             label: Some("my_effect_pipeline"),
//!             layout: Some(&pipeline_layout),
//!             vertex: wgpu::VertexState {
//!                 module: &shader,
//!                 entry_point: Some("vs_main"),
//!                 buffers: &[],  // Fullscreen triangle needs no vertex buffer
//!                 compilation_options: Default::default(),
//!             },
//!             fragment: Some(wgpu::FragmentState {
//!                 module: &shader,
//!                 entry_point: Some("fs_main"),
//!                 targets: &[Some(wgpu::ColorTargetState {
//!                     format: output_format,
//!                     blend: None,
//!                     write_mask: wgpu::ColorWrites::ALL,
//!                 })],
//!                 compilation_options: Default::default(),
//!             }),
//!             primitive: wgpu::PrimitiveState::default(),
//!             depth_stencil: None,
//!             multisample: wgpu::MultisampleState::default(),
//!             multiview_mask: None,
//!             cache: None,
//!         });
//!
//!         let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
//!             label: Some("my_effect_uniforms"),
//!             size: 8,  // time (f32) + intensity (f32)
//!             usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
//!             mapped_at_creation: false,
//!         });
//!
//!         let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
//!             mag_filter: wgpu::FilterMode::Linear,
//!             min_filter: wgpu::FilterMode::Linear,
//!             ..Default::default()
//!         });
//!
//!         Self {
//!             pipeline,
//!             bind_group_layout,
//!             bind_group: None,
//!             uniform_buffer,
//!             sampler,
//!             time: 0.0,
//!             intensity: 0.1,
//!         }
//!     }
//! }
//! ```
//!
//! ## 3. Implement PassNode
//!
//! ```ignore
//! impl PassNode<RenderInputs> for MyEffectPass {
//!     fn name(&self) -> &str { "my_effect_pass" }
//!     fn reads(&self) -> Vec<&str> { vec!["input"] }
//!     fn writes(&self) -> Vec<&str> { vec!["output"] }
//!
//!     fn prepare(&mut self, _device: &wgpu::Device, queue: &wgpu::Queue, inputs: &RenderInputs) {
//!         self.time = inputs.view.uptime_milliseconds as f32 / 1000.0;
//!         queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[
//!             self.time,
//!             self.intensity,
//!         ]));
//!     }
//!
//!     fn invalidate_bind_groups(&mut self) {
//!         self.bind_group = None;
//!     }
//!
//!     fn execute<'r, 'e>(
//!         &mut self,
//!         ctx: PassExecutionContext<'r, 'e, RenderInputs>,
//!     ) -> Result<Vec<SubGraphRunCommand<'r>>> {
//!         let input_view = ctx.get_texture_view("input")?;
//!         let (output_view, load_op, store_op) = ctx.get_color_attachment("output")?;
//!
//!         if self.bind_group.is_none() {
//!             self.bind_group = Some(ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
//!                 label: Some("my_effect_bg"),
//!                 layout: &self.bind_group_layout,
//!                 entries: &[
//!                     wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(input_view) },
//!                     wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&self.sampler) },
//!                     wgpu::BindGroupEntry { binding: 2, resource: self.uniform_buffer.as_entire_binding() },
//!                 ],
//!             }));
//!         }
//!
//!         let mut pass = ctx.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
//!             label: Some("my_effect"),
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
//! ## 4. Add to Render Graph
//!
//! ```ignore
//! let my_pass = MyEffectPass::new(device, surface_format);
//! // With the graph and the built-in target handles in hand:
//!     render_graph_add_pass(graph, Box::new(my_pass), &[
//!         ("input", targets.scene_color),
//!         ("output", targets.swapchain),
//!     ]).unwrap();
//! ```
//!
//! # Render Graph Configuration
//!
//! Extend or modify the default pipeline by adding passes against the
//! built-in target handles:
//!
//! ## Bloom + Post-Processing
//!
//! ```ignore
//! let (width, height) = (1920, 1080);
//!
//! // Create an intermediate bloom texture at half resolution
//! let bloom_texture = render_graph_add_color_texture(graph, "bloom")
//!     .format(wgpu::TextureFormat::Rgba16Float)
//!     .size(width / 2, height / 2)
//!     .clear_color(wgpu::Color::BLACK)
//!     .transient();
//!
//! // Add bloom pass
//! let bloom_pass = passes::BloomPass::new(device, width, height);
//! render_graph_pass(graph, Box::new(bloom_pass))
//!     .slot("hdr", targets.scene_color)
//!     .slot("bloom", bloom_texture)
//!     .add()
//!     .unwrap();
//!
//! // Add post-process pass combining HDR + bloom
//! let postprocess = passes::PostProcessPass::new(device, surface_format, 0.3);
//! render_graph_pass(graph, Box::new(postprocess))
//!     .slot("hdr", targets.scene_color)
//!     .slot("bloom", bloom_texture)
//!     .slot("ssao", targets.ssao)
//!     .slot("output", targets.swapchain)
//!     .add()
//!     .unwrap();
//! ```
//!
//! ## SSAO (Screen-Space Ambient Occlusion)
//!
//! ```ignore
//! // SSAO raw pass
//! let ssao_pass = passes::SsaoPass::new(device);
//! render_graph_pass(graph, Box::new(ssao_pass))
//!     .slot("depth", targets.depth)
//!     .slot("view_normals", targets.view_normals)
//!     .slot("ssao_raw", targets.ssao_raw)
//!     .add()
//!     .unwrap();
//!
//! // SSAO blur pass for soft shadows
//! let ssao_blur = passes::SsaoBlurPass::new(device);
//! render_graph_pass(graph, Box::new(ssao_blur))
//!     .slot("ssao_raw", targets.ssao_raw)
//!     .slot("depth", targets.depth)
//!     .slot("view_normals", targets.view_normals)
//!     .slot("ssao", targets.ssao)
//!     .add()
//!     .unwrap();
//! ```
//!
//! # Graphics Settings
//!
//! Shading and post-processing arrive in the `settings` field
//! ([`RenderSettings`](config::RenderSettings)), debug overlays in the
//! `debug_draw` field ([`DebugDraw`](config::DebugDraw)), and the per-frame
//! scene snapshot in the `scene` field
//! ([`RendererState`](config::RendererState)) of the frame's `RenderInputs`.
//! The caller composes these before the frame, and passes only read them.
//!
//! # Built-in Targets
//!
//! The graph declares these built-in textures:
//!
//! - `scene_color`: HDR render target (Rgba16Float)
//! - `depth`: Depth buffer (reverse-Z)
//! - `view_normals`: View-space normals for SSAO
//! - `ssao_raw`: Unblurred SSAO output
//! - `ssao`: Blurred SSAO ready for compositing
//! - `swapchain`: Final output to display

pub mod asset_id;
pub mod bounding_volume;
pub mod bundles;
pub mod config;
pub mod core;
#[cfg(feature = "text")]
pub mod font_engine;
pub mod generational_registry;
pub mod geometry;
#[cfg(feature = "grass")]
pub mod grass_config;
pub mod material;
pub mod mesh_cache;
pub mod mesh_state;
#[cfg(feature = "meshlet")]
pub mod meshlet;
pub mod particles;
pub mod procedural_meshes;
pub mod render_layer;
pub mod render_world;
#[cfg(feature = "rendergraph")]
pub mod rendergraph;
pub mod skinning;
#[cfg(feature = "terrain")]
pub mod terrain_config;
pub mod text_data;
/// Generation of textured glyph meshes from shaped text runs.
#[cfg(feature = "wgpu")]
pub mod text_mesh;
pub mod texture_data;
pub mod ui_data;
pub mod wind;

#[cfg(feature = "wgpu")]
pub mod wgpu;

#[cfg(feature = "wgpu")]
pub use core::*;
