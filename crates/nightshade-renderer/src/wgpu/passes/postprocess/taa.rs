//! Temporal anti-aliasing resolve pass.

use crate::wgpu::rendergraph::{PassExecutionContext, PassNode};
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct TaaParams {
    inv_view_proj: [[f32; 4]; 4],
    prev_view_proj: [[f32; 4]; 4],
    resolution: [f32; 2],
    history_valid: f32,
    blend: f32,
    sharpness: f32,
    use_velocity: f32,
    input_resolution: [f32; 2],
}

/// One camera's temporal history: two targets it ping-pongs between, and
/// whether either holds anything worth blending against yet.
struct CameraHistory {
    /// Retained to keep [`views`](Self::views) alive.
    _textures: Vec<wgpu::Texture>,
    views: Vec<wgpu::TextureView>,
    index: usize,
    valid: bool,
    size: (u32, u32),
}

/// Render-graph pass that blends the current frame against reprojected history
/// to resolve temporal anti-aliasing, ping-ponging between two history targets.
///
/// History is kept per camera. A frame can dispatch several cameras through this
/// pass, and they see different things: two eyes of a stereo pair, or the tiles
/// of a multi-viewport editor. Sharing one history between them resolves each
/// camera against another camera's frames, which reads as everything trailing
/// whenever anything moves.
pub struct TaaPass {
    resolve_pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    linear_sampler: wgpu::Sampler,
    point_sampler: wgpu::Sampler,
    params_buffer: wgpu::Buffer,
    format: wgpu::TextureFormat,
    /// Keyed by the camera being dispatched. `None` is the single-view path,
    /// which names no camera and needs a history of its own all the same.
    histories: std::collections::HashMap<Option<nightshade_ecs::Entity>, CameraHistory>,
    dummy_velocity_view: wgpu::TextureView,
}

impl TaaPass {
    /// Builds the resolve pipeline, samplers, and uniform buffer for output
    /// textures of `format`.
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let shader = crate::wgpu::shader_compose::compile_wgsl(
            device,
            "taa.wgsl",
            include_str!("../../shaders/taa.wgsl"),
        );

        let params = TaaParams {
            inv_view_proj: nalgebra_glm::Mat4::identity().into(),
            prev_view_proj: nalgebra_glm::Mat4::identity().into(),
            resolution: [1920.0, 1080.0],
            history_valid: 0.0,
            blend: 0.12,
            sharpness: 0.5,
            use_velocity: 0.0,
            input_resolution: [1920.0, 1080.0],
        };
        let params_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("TAA Params Buffer"),
            contents: bytemuck::cast_slice(&[params]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("TAA Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Depth,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 6,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("TAA Pipeline Layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let target = Some(wgpu::ColorTargetState {
            format,
            blend: None,
            write_mask: wgpu::ColorWrites::ALL,
        });
        let resolve_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("TAA Resolve Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vertex_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fragment_main"),
                targets: &[target.clone(), target],
                compilation_options: Default::default(),
            }),
            multiview_mask: None,
            cache: None,
        });

        let linear_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("TAA Linear Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });
        let point_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("TAA Point Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        let dummy_velocity = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("TAA Dummy Velocity"),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rg16Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let dummy_velocity_view =
            dummy_velocity.create_view(&wgpu::TextureViewDescriptor::default());

        Self {
            resolve_pipeline,
            bind_group_layout,
            linear_sampler,
            point_sampler,
            params_buffer,
            format,
            histories: std::collections::HashMap::new(),
            dummy_velocity_view,
        }
    }

    /// Ensures `camera` has a history pair at this size, rebuilding it when the
    /// size changed and starting it invalid so the first frame after a resize
    /// blends against nothing.
    fn ensure_history(
        &mut self,
        device: &wgpu::Device,
        camera: Option<nightshade_ecs::Entity>,
        width: u32,
        height: u32,
    ) {
        if self
            .histories
            .get(&camera)
            .is_some_and(|history| history.size == (width, height))
        {
            return;
        }

        let mut textures = Vec::new();
        let mut views = Vec::new();
        for _ in 0..2 {
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("TAA History Texture"),
                size: wgpu::Extent3d {
                    width: width.max(1),
                    height: height.max(1),
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: self.format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });
            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
            textures.push(texture);
            views.push(view);
        }

        self.histories.insert(
            camera,
            CameraHistory {
                _textures: textures,
                views,
                index: 0,
                valid: false,
                size: (width, height),
            },
        );
    }

    /// Drops history for cameras the frame no longer dispatches, so a closed
    /// viewport or a despawned eye does not hold its targets forever.
    pub fn retain_cameras(&mut self, live: &std::collections::HashSet<nightshade_ecs::Entity>) {
        self.histories
            .retain(|camera, _| camera.is_none_or(|entity| live.contains(&entity)));
    }
}

impl PassNode<crate::wgpu::render_configs::RenderInputs> for TaaPass {
    fn name(&self) -> &str {
        "taa_pass"
    }

    fn reads(&self) -> Vec<&str> {
        vec!["input", "depth"]
    }

    fn optional_reads(&self) -> Vec<&str> {
        vec!["velocity"]
    }

    fn writes(&self) -> Vec<&str> {
        vec!["output"]
    }

    fn execute<'r, 'e>(
        &mut self,
        context: PassExecutionContext<'r, 'e, crate::wgpu::render_configs::RenderInputs>,
    ) -> crate::wgpu::rendergraph::Result<Vec<crate::wgpu::rendergraph::SubGraphRunCommand<'r>>>
    {
        let (width, height) = context
            .get_texture("output")
            .map(|texture| (texture.width(), texture.height()))
            .unwrap_or((1920, 1080));

        let camera = context.configs.view.active_camera;
        self.ensure_history(context.device, camera, width, height);

        // Reproject with the unjittered view-projection the velocity producers
        // use (renderer_state tracks current and previous), so the depth
        // fallback and the motion-vector path land in the same previous space.
        let view_projection = nalgebra_glm::Mat4::from(context.configs.scene.view_projection);
        let inv_view_proj = view_projection
            .try_inverse()
            .unwrap_or_else(nalgebra_glm::Mat4::identity);
        let prev_view_proj = context.configs.scene.prev_view_projection;

        let taa_enabled = context.configs.settings.taa_enabled;
        let taa_blend = context.configs.settings.taa_blend;
        let taa_sharpness = context.configs.settings.taa_sharpness;
        let velocity_slot = context.get_texture_view("velocity").ok();
        let has_velocity = velocity_slot.is_some();
        let velocity_view = velocity_slot.unwrap_or(&self.dummy_velocity_view);
        let (read_index, write_index, history_valid) = self
            .histories
            .get(&camera)
            .map(|history| (history.index, 1 - history.index, history.valid))
            .unwrap_or((0, 1, false));
        let valid = taa_enabled && history_valid;
        let (input_width, input_height) = context
            .get_texture("input")
            .map(|texture| (texture.width(), texture.height()))
            .unwrap_or((width, height));
        let params = TaaParams {
            inv_view_proj: inv_view_proj.into(),
            prev_view_proj,
            resolution: [width.max(1) as f32, height.max(1) as f32],
            history_valid: if valid { 1.0 } else { 0.0 },
            blend: taa_blend,
            sharpness: taa_sharpness,
            use_velocity: if has_velocity && valid { 1.0 } else { 0.0 },
            input_resolution: [input_width.max(1) as f32, input_height.max(1) as f32],
        };
        context
            .queue
            .write_buffer(&self.params_buffer, 0, bytemuck::cast_slice(&[params]));

        let input_view = context.get_texture_view("input")?;
        let (depth_view, _, _) = context.get_depth_attachment("depth")?;

        let bind_group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("TAA Bind Group"),
                layout: &self.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(input_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(
                            &self.histories[&camera].views[read_index],
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(depth_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::Sampler(&self.linear_sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: wgpu::BindingResource::Sampler(&self.point_sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 5,
                        resource: self.params_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 6,
                        resource: wgpu::BindingResource::TextureView(velocity_view),
                    },
                ],
            });

        let (output_view, output_load_op, output_store_op) =
            context.get_color_attachment("output")?;

        {
            let mut render_pass = context
                .encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("TAA Resolve Pass"),
                    color_attachments: &[
                        Some(wgpu::RenderPassColorAttachment {
                            view: output_view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: output_load_op,
                                store: output_store_op,
                            },
                            depth_slice: None,
                        }),
                        Some(wgpu::RenderPassColorAttachment {
                            view: &self.histories[&camera].views[write_index],
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                                store: wgpu::StoreOp::Store,
                            },
                            depth_slice: None,
                        }),
                    ],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                });

            render_pass.set_pipeline(&self.resolve_pipeline);
            render_pass.set_bind_group(0, &bind_group, &[]);
            render_pass.draw(0..3, 0..1);
        }

        if let Some(history) = self.histories.get_mut(&camera) {
            history.index = write_index;
            history.valid = true;
        }

        Ok(context.into_sub_graph_commands())
    }
}
