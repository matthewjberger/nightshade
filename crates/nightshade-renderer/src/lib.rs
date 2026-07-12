//! Rendering system using WebGPU (wgpu).
//!
//! The render module provides:
//!
//! - **Render Graph**: Declarative pass-based rendering with automatic resource management
//! - **PBR Materials**: Physically-based rendering with metallic-roughness workflow
//! - **Shadow Mapping**: Cascaded shadow maps for directional lights
//! - **Post-Processing**: Bloom, SSAO, depth of field, tonemapping, color grading
//! - **GPU Particles**: Compute shader particle simulation and billboard rendering
//! - **Text Rendering**: Signed distance field text with GPU atlasing
//!
//! Most rendering is automatic based on ECS components. Custom passes can be added
//! via `State::configure_render_graph`.
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
//! impl PassNode<World> for MyEffectPass {
//!     fn name(&self) -> &str { "my_effect_pass" }
//!     fn reads(&self) -> Vec<&str> { vec!["input"] }
//!     fn writes(&self) -> Vec<&str> { vec!["output"] }
//!
//!     fn prepare(&mut self, _device: &wgpu::Device, queue: &wgpu::Queue, world: &World) {
//!         self.time = world.resources.window.timing.uptime_milliseconds as f32 / 1000.0;
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
//!         ctx: PassExecutionContext<'r, 'e, World>,
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
//! fn configure_render_graph(
//!     &mut self,
//!     graph: &mut RenderGraph<World>,
//!     device: &wgpu::Device,
//!     surface_format: wgpu::TextureFormat,
//!     resources: RenderResources,
//! ) {
//!     let my_pass = MyEffectPass::new(device, surface_format);
//!     render_graph_add_pass(graph, Box::new(my_pass), &[
//!         ("input", resources.scene_color),
//!         ("output", resources.swapchain),
//!     ]).unwrap();
//! }
//! ```
//!
//! # Render Graph Configuration
//!
//! Override the default render pipeline by implementing `configure_render_graph`:
//!
//! ## Bloom + Post-Processing
//!
//! ```ignore
//! fn configure_render_graph(
//!     &mut self,
//!     graph: &mut RenderGraph<World>,
//!     device: &wgpu::Device,
//!     surface_format: wgpu::TextureFormat,
//!     resources: RenderResources,
//! ) {
//!     let (width, height) = (1920, 1080);
//!
//!     // Create intermediate bloom texture at half resolution
//!     let bloom_texture = render_graph_add_color_texture(graph, "bloom")
//!         .format(wgpu::TextureFormat::Rgba16Float)
//!         .size(width / 2, height / 2)
//!         .clear_color(wgpu::Color::BLACK)
//!         .transient();
//!
//!     // Add bloom pass
//!     let bloom_pass = passes::BloomPass::new(device, width, height);
//!     render_graph_pass(graph, Box::new(bloom_pass))
//!         .read("hdr", resources.scene_color)
//!         .write("bloom", bloom_texture);
//!
//!     // Add post-process pass combining HDR + bloom
//!     let postprocess = passes::PostProcessPass::new(device, surface_format, 0.3);
//!     render_graph_pass(graph, Box::new(postprocess))
//!         .read("hdr", resources.scene_color)
//!         .read("bloom", bloom_texture)
//!         .read("ssao", resources.ssao)
//!         .write("output", resources.swapchain);
//! }
//! ```
//!
//! ## SSAO (Screen-Space Ambient Occlusion)
//!
//! ```ignore
//! fn configure_render_graph(...) {
//!     // SSAO raw pass
//!     let ssao_pass = passes::SsaoPass::new(device);
//!     render_graph_pass(graph, Box::new(ssao_pass))
//!         .read("depth", resources.depth)
//!         .read("view_normals", resources.view_normals)
//!         .write("ssao_raw", resources.ssao_raw);
//!
//!     // SSAO blur pass for soft shadows
//!     let ssao_blur = passes::SsaoBlurPass::new(device);
//!     render_graph_pass(graph, Box::new(ssao_blur))
//!         .read("ssao_raw", resources.ssao_raw)
//!         .read("depth", resources.depth)
//!         .read("view_normals", resources.view_normals)
//!         .write("ssao", resources.ssao);
//! }
//! ```
//!
//! # Graphics Settings
//!
//! Shading and post-processing live on `world.resources.render_settings`,
//! debug overlays on `world.resources.debug_draw`:
//!
//! ```ignore
//! let render = &mut world.resources.render_settings;
//!
//! // Atmosphere/sky
//! render.atmosphere = Atmosphere::Sky;        // Procedural sky
//! render.atmosphere = Atmosphere::None;       // Solid clear color
//! render.atmosphere = Atmosphere::Hdr;        // HDR skybox
//!
//! // Post-processing
//! render.bloom_enabled = true;
//! render.bloom_intensity = 0.3;               // 0.0 - 1.0
//! render.ssao_enabled = true;
//! render.ssao_radius = 0.5;                   // World units
//! render.ssao_intensity = 1.0;
//!
//! // Visual aids
//! world.resources.debug_draw.show_grid = true;       // Ground grid
//! world.resources.renderer_state.show_cursor = true; // OS cursor visibility
//! ```
//!
//! # Available RenderResources
//!
//! The `RenderResources` struct provides access to built-in textures:
//!
//! - `scene_color`: HDR render target (Rgba16Float)
//! - `depth`: Depth buffer (reverse-Z)
//! - `view_normals`: View-space normals for SSAO
//! - `ssao_raw`: Unblurred SSAO output
//! - `ssao`: Blurred SSAO ready for compositing
//! - `swapchain`: Final output to display

pub mod asset_id;
pub mod bounding_volume;
pub mod config;
pub mod core;
pub mod generational_registry;
pub mod geometry;
#[cfg(feature = "grass")]
pub mod grass_config;
pub mod material;
pub mod mesh_cache;
pub mod particles;
pub mod procedural_meshes;
pub mod render_layer;
pub mod skinning;
#[cfg(feature = "terrain")]
pub mod terrain_config;
pub mod text_data;
pub mod ui_data;
pub mod wind;

#[cfg(feature = "wgpu")]
pub mod wgpu;

pub use core::*;
