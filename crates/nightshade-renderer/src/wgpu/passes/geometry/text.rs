use crate::text_data::TextVertex;
use crate::wgpu::rendergraph::{PassExecutionContext, PassNode};
use nalgebra_glm::{Mat4, Vec4};

/// Render pass drawing signed-distance-field text, screen-space or billboarded.
pub struct TextPass {
    /// Pipeline drawing text glyph quads.
    pub render_pipeline: wgpu::RenderPipeline,
    /// Layout for the glyph atlas texture, sampler, and character colors.
    pub texture_bind_group_layout: wgpu::BindGroupLayout,
    /// Layout for the per-instance text uniform.
    pub text_bind_group_layout: wgpu::BindGroupLayout,

    /// Glyph quad vertices for all instances.
    pub vertex_buffer: wgpu::Buffer,
    /// Glyph quad indices for all instances.
    pub index_buffer: wgpu::Buffer,
    /// Scene view and projection uniform.
    pub uniform_buffer: wgpu::Buffer,
    /// Per-instance text uniforms, addressed by dynamic offset.
    pub text_uniform_buffer: wgpu::Buffer,
    /// Per-character color overrides read by the shader.
    pub character_color_buffer: wgpu::Buffer,

    /// Bind group for the scene uniform.
    pub uniform_bind_group: wgpu::BindGroup,
    /// Bind groups for the per-instance text uniform buffer.
    pub text_bind_groups: Vec<wgpu::BindGroup>,
    /// Bind group for the glyph atlas, absent until an atlas is set.
    pub atlas_bind_group: Option<wgpu::BindGroup>,

    /// Text instances collected for the current frame.
    pub text_instances: Vec<TextInstance>,
    scratch_vertices: Vec<TextVertex>,
    scratch_indices: Vec<u32>,
    scratch_character_colors: Vec<[f32; 4]>,

    vertex_buffer_capacity: usize,
    index_buffer_capacity: usize,
    uniform_buffer_instances: usize,
    character_color_buffer_capacity: usize,

    cached_atlas_view: Option<wgpu::TextureView>,
    cached_sampler: Option<wgpu::Sampler>,
}

/// One text mesh queued for drawing this frame.
#[derive(Clone)]
pub struct TextInstance {
    /// First vertex of this instance in the shared vertex buffer.
    pub vertex_offset: u32,
    /// First index of this instance in the shared index buffer.
    pub index_offset: u32,
    /// Number of indices this instance draws.
    pub index_count: u32,
    /// World transform of the text.
    pub transform: Mat4,
    /// Fill color as linear RGBA.
    pub color: Vec4,
    /// Outline color as linear RGBA.
    pub outline_color: Vec4,
    /// Outline width in signed-distance units.
    pub outline_width: f32,
    /// Edge smoothing width in signed-distance units.
    pub smoothing: f32,
    /// Whether the text always faces the camera.
    pub billboard: bool,
}

/// Per-instance text uniform matching the shader layout.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TextUniforms {
    /// Model transform applied to the glyph quads.
    pub model: Mat4,
    /// Fill color as linear RGBA.
    pub color: Vec4,
    /// Outline color as linear RGBA.
    pub outline_color: Vec4,
    /// Outline width in signed-distance units.
    pub outline_width: f32,
    /// Edge smoothing width in signed-distance units.
    pub smoothing: f32,
    /// Padding for 16-byte alignment.
    pub _padding: [f32; 2],
}

/// Scene camera uniform shared by all text instances.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SceneUniforms {
    /// View matrix.
    pub view: Mat4,
    /// Projection matrix.
    pub projection: Mat4,
    /// Camera position in world space, w unused.
    pub camera_position: Vec4,
}

const INITIAL_CHARACTER_COLOR_CAPACITY: usize = 10000;

impl TextPass {
    /// Creates the pass, its pipeline, and the initial glyph buffers.
    pub fn new(
        device: &wgpu::Device,
        color_format: wgpu::TextureFormat,
        depth_format: wgpu::TextureFormat,
    ) -> Self {
        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Text Uniform Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let text_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Text Instance Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: wgpu::BufferSize::new(
                            std::mem::size_of::<TextUniforms>() as u64,
                        ),
                    },
                    count: None,
                }],
            });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Text Texture Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Text Pipeline Layout"),
            bind_group_layouts: &[
                Some(&uniform_bind_group_layout),
                Some(&text_bind_group_layout),
                Some(&texture_bind_group_layout),
            ],
            immediate_size: 0,
        });

        let shader = crate::wgpu::shader_compose::compile_wgsl(
            device,
            "Text Shader",
            include_str!("../../shaders/text.wgsl"),
        );

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Text Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<TextVertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            offset: 0,
                            shader_location: 0,
                            format: wgpu::VertexFormat::Float32x3,
                        },
                        wgpu::VertexAttribute {
                            offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                            shader_location: 1,
                            format: wgpu::VertexFormat::Float32x2,
                        },
                        wgpu::VertexAttribute {
                            offset: std::mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
                            shader_location: 2,
                            format: wgpu::VertexFormat::Uint32,
                        },
                    ],
                }],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: depth_format,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::Greater),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: color_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            multiview_mask: None,
            cache: None,
        });

        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Text Vertex Buffer"),
            size: (std::mem::size_of::<TextVertex>() * 10000) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Text Index Buffer"),
            size: (std::mem::size_of::<u32>() * 30000) as u64,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Text Scene Uniform Buffer"),
            size: std::mem::size_of::<SceneUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let uniform_size = std::mem::size_of::<TextUniforms>();
        let alignment = device.limits().min_uniform_buffer_offset_alignment as usize;
        let aligned_size = (uniform_size + alignment - 1) & !(alignment - 1);
        let text_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Text Instance Uniform Buffer"),
            size: (aligned_size * 100) as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let character_color_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Text Character Color Buffer"),
            size: (std::mem::size_of::<[f32; 4]>() * INITIAL_CHARACTER_COLOR_CAPACITY) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Text Uniform Bind Group"),
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let text_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Text Instance Bind Group"),
            layout: &text_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &text_uniform_buffer,
                    offset: 0,
                    size: wgpu::BufferSize::new(aligned_size as u64),
                }),
            }],
        });

        Self {
            render_pipeline,
            texture_bind_group_layout,
            text_bind_group_layout,
            vertex_buffer,
            index_buffer,
            uniform_buffer,
            text_uniform_buffer,
            character_color_buffer,
            uniform_bind_group,
            text_bind_groups: vec![text_bind_group],
            atlas_bind_group: None,
            text_instances: Vec::new(),
            scratch_vertices: Vec::new(),
            scratch_indices: Vec::new(),
            scratch_character_colors: Vec::new(),
            vertex_buffer_capacity: 10000,
            index_buffer_capacity: 30000,
            uniform_buffer_instances: 100,
            character_color_buffer_capacity: INITIAL_CHARACTER_COLOR_CAPACITY,
            cached_atlas_view: None,
            cached_sampler: None,
        }
    }

    fn ensure_vertex_buffer_capacity(&mut self, device: &wgpu::Device, required: usize) {
        if required > self.vertex_buffer_capacity {
            let new_capacity = (required * 2).max(1000);
            self.vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Text Vertex Buffer (Resized)"),
                size: (std::mem::size_of::<TextVertex>() * new_capacity) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.vertex_buffer_capacity = new_capacity;
        }
    }

    fn ensure_index_buffer_capacity(&mut self, device: &wgpu::Device, required: usize) {
        if required > self.index_buffer_capacity {
            let new_capacity = (required * 2).max(3000);
            self.index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Text Index Buffer (Resized)"),
                size: (std::mem::size_of::<u32>() * new_capacity) as u64,
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.index_buffer_capacity = new_capacity;
        }
    }

    fn ensure_uniform_buffer_capacity(&mut self, device: &wgpu::Device, required: usize) {
        if required > self.uniform_buffer_instances {
            let uniform_size = std::mem::size_of::<TextUniforms>();
            let alignment = device.limits().min_uniform_buffer_offset_alignment as usize;
            let aligned_size = (uniform_size + alignment - 1) & !(alignment - 1);
            let new_capacity = (required * 2).max(10);

            self.text_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Text Instance Uniform Buffer (Resized)"),
                size: (aligned_size * new_capacity) as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });

            let text_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Text Instance Bind Group (Resized)"),
                layout: &self.text_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &self.text_uniform_buffer,
                        offset: 0,
                        size: wgpu::BufferSize::new(aligned_size as u64),
                    }),
                }],
            });

            self.text_bind_groups = vec![text_bind_group];
            self.uniform_buffer_instances = new_capacity;
        }
    }

    fn ensure_character_color_buffer_capacity(&mut self, device: &wgpu::Device, required: usize) {
        if required > self.character_color_buffer_capacity {
            let new_capacity = (required * 2).max(1000);
            self.character_color_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Text Character Color Buffer (Resized)"),
                size: (std::mem::size_of::<[f32; 4]>() * new_capacity) as u64,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.character_color_buffer_capacity = new_capacity;
            self.rebuild_atlas_bind_group(device);
        }
    }

    /// Points the pass at a new glyph atlas view and rebuilds its bind group.
    pub fn update_glyph_atlas(&mut self, device: &wgpu::Device, atlas_view: &wgpu::TextureView) {
        self.cached_atlas_view = Some(atlas_view.clone());
        self.rebuild_atlas_bind_group(device);
    }

    /// Records the text draw for every collected instance.
    pub fn record(&self, render_pass: &mut wgpu::RenderPass<'_>, device: &wgpu::Device) {
        if self.text_instances.is_empty() {
            return;
        }

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

        for (i, instance) in self.text_instances.iter().enumerate() {
            if i >= self.uniform_buffer_instances {
                tracing::warn!(
                    "Skipping text instance {} - exceeds uniform buffer capacity of {}",
                    i,
                    self.uniform_buffer_instances
                );
                break;
            }

            let index_end = instance.index_offset.saturating_add(instance.index_count);
            if instance.vertex_offset >= self.vertex_buffer_capacity as u32
                || index_end > self.index_buffer_capacity as u32
            {
                tracing::warn!("Skipping text instance with out-of-bounds indices");
                continue;
            }

            let uniform_size = std::mem::size_of::<TextUniforms>();
            let alignment = device.limits().min_uniform_buffer_offset_alignment as usize;
            let aligned_size = (uniform_size + alignment - 1) & !(alignment - 1);
            let offset = (i as u32) * (aligned_size as u32);
            render_pass.set_bind_group(1, &self.text_bind_groups[0], &[offset]);

            if let Some(atlas_bind_group) = self.atlas_bind_group.as_ref() {
                render_pass.set_bind_group(2, atlas_bind_group, &[]);
            } else {
                continue;
            }

            render_pass.draw_indexed(instance.index_offset..index_end, 0, 0..1);
        }
    }

    fn rebuild_atlas_bind_group(&mut self, device: &wgpu::Device) {
        let Some(atlas_view) = self.cached_atlas_view.as_ref() else {
            self.atlas_bind_group = None;
            return;
        };

        let sampler = self.cached_sampler.get_or_insert_with(|| {
            device.create_sampler(&wgpu::SamplerDescriptor {
                label: Some("Glyph Atlas Sampler"),
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::MipmapFilterMode::Linear,
                ..Default::default()
            })
        });

        self.atlas_bind_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Text Glyph Atlas Bind Group"),
            layout: &self.texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(atlas_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.character_color_buffer.as_entire_binding(),
                },
            ],
        }));
    }
}

impl PassNode<crate::wgpu::render_configs::RenderInputs> for TextPass {
    fn name(&self) -> &'static str {
        "text_pass"
    }

    fn reads(&self) -> Vec<&str> {
        vec![]
    }

    fn writes(&self) -> Vec<&str> {
        vec![]
    }

    fn reads_writes(&self) -> Vec<&str> {
        vec!["color", "depth"]
    }

    fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        configs: &crate::wgpu::render_configs::RenderInputs,
    ) {
        self.text_instances.clear();
        self.scratch_vertices.clear();
        self.scratch_indices.clear();
        self.scratch_character_colors.clear();
        let mut character_color_offset = 0u32;

        let view_matrix = if let Some(camera_matrices) = configs.scene.render_view.as_ref() {
            let camera_position = nalgebra_glm::vec4(0.0, 0.0, 0.0, 1.0);
            let scene_uniforms = SceneUniforms {
                view: camera_matrices.view,
                projection: camera_matrices.projection,
                camera_position,
            };
            queue.write_buffer(
                &self.uniform_buffer,
                0,
                bytemuck::cast_slice(&[scene_uniforms]),
            );
            Some(camera_matrices.view)
        } else {
            None
        };

        for (_, render_text) in configs
            .scene_world
            .query_ref::<&crate::config::RenderText>()
            .iter()
        {
            let global_transform = render_text.transform;
            let character_colors = render_text.character_colors.as_ref();

            {
                let mesh = &render_text.mesh;
                let vertex_offset = self.scratch_vertices.len() as u32;
                let index_offset = self.scratch_indices.len() as u32;

                for &index in &mesh.indices {
                    self.scratch_indices.push(index + vertex_offset);
                }

                let char_count = render_text.char_count;

                for vertex in &mesh.vertices {
                    let adjusted_vertex = TextVertex {
                        position: vertex.position,
                        tex_coords: vertex.tex_coords,
                        character_index: vertex.character_index + character_color_offset,
                        _padding: 0,
                    };
                    self.scratch_vertices.push(adjusted_vertex);
                }

                for character_index in 0..char_count {
                    let color = if let Some(colors) = character_colors {
                        colors
                            .get(character_index)
                            .and_then(|c| *c)
                            .map(|c| [c.x, c.y, c.z, c.w])
                            .unwrap_or([0.0, 0.0, 0.0, 0.0])
                    } else {
                        [0.0, 0.0, 0.0, 0.0]
                    };
                    self.scratch_character_colors.push(color);
                }

                character_color_offset += char_count as u32;

                self.text_instances.push(TextInstance {
                    vertex_offset,
                    index_offset,
                    index_count: mesh.indices.len() as u32,
                    transform: global_transform,
                    color: render_text.color,
                    outline_color: render_text.outline_color,
                    outline_width: render_text.outline_width,
                    smoothing: render_text.smoothing,
                    billboard: render_text.billboard,
                });
            }
        }

        if self.scratch_character_colors.is_empty() {
            self.scratch_character_colors.push([0.0, 0.0, 0.0, 0.0]);
        }

        self.ensure_character_color_buffer_capacity(device, self.scratch_character_colors.len());
        queue.write_buffer(
            &self.character_color_buffer,
            0,
            bytemuck::cast_slice(&self.scratch_character_colors),
        );

        if !self.scratch_vertices.is_empty() {
            self.ensure_vertex_buffer_capacity(device, self.scratch_vertices.len());
            queue.write_buffer(
                &self.vertex_buffer,
                0,
                bytemuck::cast_slice(&self.scratch_vertices),
            );
        }

        if !self.scratch_indices.is_empty() {
            self.ensure_index_buffer_capacity(device, self.scratch_indices.len());
            queue.write_buffer(
                &self.index_buffer,
                0,
                bytemuck::cast_slice(&self.scratch_indices),
            );
        }

        let uniform_size = std::mem::size_of::<TextUniforms>();
        let alignment = device.limits().min_uniform_buffer_offset_alignment as usize;
        let aligned_size = (uniform_size + alignment - 1) & !(alignment - 1);

        self.ensure_uniform_buffer_capacity(device, self.text_instances.len());

        for (i, instance) in self.text_instances.iter().enumerate() {
            if i >= self.uniform_buffer_instances {
                break;
            }

            let model = if instance.billboard {
                if let Some(view) = view_matrix {
                    let position = nalgebra_glm::vec3(
                        instance.transform[(0, 3)],
                        instance.transform[(1, 3)],
                        instance.transform[(2, 3)],
                    );

                    let scale_x = nalgebra_glm::length(&nalgebra_glm::vec3(
                        instance.transform[(0, 0)],
                        instance.transform[(1, 0)],
                        instance.transform[(2, 0)],
                    ));
                    let scale_y = nalgebra_glm::length(&nalgebra_glm::vec3(
                        instance.transform[(0, 1)],
                        instance.transform[(1, 1)],
                        instance.transform[(2, 1)],
                    ));
                    let scale_z = nalgebra_glm::length(&nalgebra_glm::vec3(
                        instance.transform[(0, 2)],
                        instance.transform[(1, 2)],
                        instance.transform[(2, 2)],
                    ));

                    let right = nalgebra_glm::vec3(view[(0, 0)], view[(0, 1)], view[(0, 2)]);
                    let up = nalgebra_glm::vec3(view[(1, 0)], view[(1, 1)], view[(1, 2)]);
                    let forward = nalgebra_glm::vec3(-view[(2, 0)], -view[(2, 1)], -view[(2, 2)]);

                    #[rustfmt::skip]
                    let billboard_transform = Mat4::new(
                        right.x * scale_x, up.x * scale_y, forward.x * scale_z, position.x,
                        right.y * scale_x, up.y * scale_y, forward.y * scale_z, position.y,
                        right.z * scale_x, up.z * scale_y, forward.z * scale_z, position.z,
                        0.0, 0.0, 0.0, 1.0,
                    );

                    billboard_transform
                } else {
                    instance.transform
                }
            } else {
                instance.transform
            };

            let text_uniforms = TextUniforms {
                model,
                color: instance.color,
                outline_color: instance.outline_color,
                outline_width: instance.outline_width,
                smoothing: instance.smoothing,
                _padding: [0.0; 2],
            };

            let offset = i * aligned_size;
            queue.write_buffer(
                &self.text_uniform_buffer,
                offset as u64,
                bytemuck::cast_slice(&[text_uniforms]),
            );
        }
    }

    fn execute<'r, 'e>(
        &mut self,
        context: PassExecutionContext<'r, 'e, crate::wgpu::render_configs::RenderInputs>,
    ) -> Result<
        Vec<crate::wgpu::rendergraph::SubGraphRunCommand<'r>>,
        crate::wgpu::rendergraph::RenderGraphError,
    > {
        let (color_view, color_load, color_store) = context.get_color_attachment("color")?;
        let (depth_view, depth_load, depth_store) = context.get_depth_attachment("depth")?;

        let mut render_pass = context
            .encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Text Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: color_view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: color_load,
                        store: color_store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: depth_load,
                        store: depth_store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

        self.record(&mut render_pass, context.device);

        drop(render_pass);

        Ok(context.into_sub_graph_commands())
    }
}
