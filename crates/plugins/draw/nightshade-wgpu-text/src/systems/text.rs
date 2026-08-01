use nalgebra_glm::{Mat4, Vec4};
use nightshade_text::contract::data::text_data::TextVertex;
use nightshade_wgpu::backend::PassBuild;
use nightshade_wgpu::backend::build;
use nightshade_wgpu::wgpu;

pub struct TextPass {
    pub render_pipeline: wgpu::RenderPipeline,
    pub texture_bind_group_layout: wgpu::BindGroupLayout,
    pub text_bind_group_layout: wgpu::BindGroupLayout,

    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub uniform_buffer: wgpu::Buffer,
    pub text_uniform_buffer: wgpu::Buffer,
    pub character_color_buffer: wgpu::Buffer,

    pub uniform_bind_group: wgpu::BindGroup,
    pub text_bind_groups: Vec<wgpu::BindGroup>,
    pub atlas_bind_group: Option<wgpu::BindGroup>,

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

#[derive(Clone)]
pub struct TextInstance {
    pub vertex_offset: u32,
    pub index_offset: u32,
    pub index_count: u32,
    pub transform: Mat4,
    pub color: Vec4,
    pub outline_color: Vec4,
    pub outline_width: f32,
    pub smoothing: f32,
    pub billboard: bool,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TextUniforms {
    pub model: Mat4,
    pub color: Vec4,
    pub outline_color: Vec4,
    pub outline_width: f32,
    pub smoothing: f32,
    pub padding: [f32; 2],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SceneUniforms {
    pub view: Mat4,
    pub projection: Mat4,
    pub camera_position: Vec4,
}

const INITIAL_CHARACTER_COLOR_CAPACITY: usize = 10000;

impl TextPass {
    pub fn new(
        build: PassBuild,
        color_format: wgpu::TextureFormat,
        depth_format: wgpu::TextureFormat,
    ) -> Self {
        let PassBuild {
            device, composer, ..
        } = build;
        let uniform_bind_group_layout = build::bind_layout(
            device,
            "Text Uniform Bind Group Layout",
            &[build::uniform_at(0, wgpu::ShaderStages::VERTEX)],
        );

        let text_bind_group_layout = build::bind_layout(
            device,
            "Text Instance Bind Group Layout",
            &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: wgpu::BufferSize::new(
                        std::mem::size_of::<TextUniforms>() as u64
                    ),
                },
                count: None,
            }],
        );

        let texture_bind_group_layout = build::bind_layout(
            device,
            "Text Texture Bind Group Layout",
            &[
                build::painted_at(0),
                build::filtering_at(1),
                build::storage_at(2, false, wgpu::ShaderStages::FRAGMENT),
            ],
        );

        let pipeline_layout = build::pipeline_layout(
            device,
            "Text Pipeline Layout",
            &[
                &uniform_bind_group_layout,
                &text_bind_group_layout,
                &texture_bind_group_layout,
            ],
        );

        let shader =
            composer.compile_wgsl(device, "Text Shader", include_str!("../shaders/text.wgsl"));

        let render_pipeline = build::Pipe::new(device, "Text Render Pipeline", &shader)
            .layout(&pipeline_layout)
            .buffers(&[wgpu::VertexBufferLayout {
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
                    wgpu::VertexAttribute {
                        offset: std::mem::size_of::<[f32; 6]>() as wgpu::BufferAddress,
                        shader_location: 3,
                        format: wgpu::VertexFormat::Uint32,
                    },
                ],
            }])
            .target(wgpu::ColorTargetState {
                format: color_format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })
            .depth(wgpu::DepthStencilState {
                format: depth_format,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::Greater),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            })
            .vertex("vs_main")
            .fragment("fs_main")
            .make();

        let vertex_buffer = build::buffer(
            device,
            "Text Vertex Buffer",
            (std::mem::size_of::<TextVertex>() * 10000) as u64,
            wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        );

        let index_buffer = build::buffer(
            device,
            "Text Index Buffer",
            (std::mem::size_of::<u32>() * 30000) as u64,
            wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        );

        let uniform_buffer = build::uniform(
            device,
            "Text Scene Uniform Buffer",
            std::mem::size_of::<SceneUniforms>() as u64,
        );

        let uniform_size = std::mem::size_of::<TextUniforms>();
        let alignment = device.limits().min_uniform_buffer_offset_alignment as usize;
        let aligned_size = (uniform_size + alignment - 1) & !(alignment - 1);
        let text_uniform_buffer = build::uniform(
            device,
            "Text Instance Uniform Buffer",
            (aligned_size * 100) as u64,
        );

        let character_color_buffer = build::storage(
            device,
            "Text Character Color Buffer",
            (std::mem::size_of::<[f32; 4]>() * INITIAL_CHARACTER_COLOR_CAPACITY) as u64,
        );

        let uniform_bind_group = build::binding(
            device,
            "Text Uniform Bind Group",
            &uniform_bind_group_layout,
            vec![uniform_buffer.as_entire_binding()],
        );

        let text_bind_group = build::binding(
            device,
            "Text Instance Bind Group",
            &text_bind_group_layout,
            vec![wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                buffer: &text_uniform_buffer,
                offset: 0,
                size: wgpu::BufferSize::new(aligned_size as u64),
            })],
        );

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
            self.vertex_buffer = build::buffer(
                device,
                "Text Vertex Buffer (Resized)",
                (std::mem::size_of::<TextVertex>() * new_capacity) as u64,
                wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            );
            self.vertex_buffer_capacity = new_capacity;
        }
    }

    fn ensure_index_buffer_capacity(&mut self, device: &wgpu::Device, required: usize) {
        if required > self.index_buffer_capacity {
            let new_capacity = (required * 2).max(3000);
            self.index_buffer = build::buffer(
                device,
                "Text Index Buffer (Resized)",
                (std::mem::size_of::<u32>() * new_capacity) as u64,
                wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            );
            self.index_buffer_capacity = new_capacity;
        }
    }

    fn ensure_uniform_buffer_capacity(&mut self, device: &wgpu::Device, required: usize) {
        if required > self.uniform_buffer_instances {
            let uniform_size = std::mem::size_of::<TextUniforms>();
            let alignment = device.limits().min_uniform_buffer_offset_alignment as usize;
            let aligned_size = (uniform_size + alignment - 1) & !(alignment - 1);
            let new_capacity = (required * 2).max(10);

            self.text_uniform_buffer = build::uniform(
                device,
                "Text Instance Uniform Buffer (Resized)",
                (aligned_size * new_capacity) as u64,
            );

            let text_bind_group = build::binding(
                device,
                "Text Instance Bind Group (Resized)",
                &self.text_bind_group_layout,
                vec![wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &self.text_uniform_buffer,
                    offset: 0,
                    size: wgpu::BufferSize::new(aligned_size as u64),
                })],
            );

            self.text_bind_groups = vec![text_bind_group];
            self.uniform_buffer_instances = new_capacity;
        }
    }

    fn ensure_character_color_buffer_capacity(&mut self, device: &wgpu::Device, required: usize) {
        if required > self.character_color_buffer_capacity {
            let new_capacity = (required * 2).max(1000);
            self.character_color_buffer = build::storage(
                device,
                "Text Character Color Buffer (Resized)",
                (std::mem::size_of::<[f32; 4]>() * new_capacity) as u64,
            );
            self.character_color_buffer_capacity = new_capacity;
            self.rebuild_atlas_bind_group(device);
        }
    }

    pub fn update_glyph_atlas(&mut self, device: &wgpu::Device, atlas_view: &wgpu::TextureView) {
        self.cached_atlas_view = Some(atlas_view.clone());
        self.rebuild_atlas_bind_group(device);
    }

    pub fn release_glyph_atlas(&mut self) {
        self.cached_atlas_view = None;
        self.atlas_bind_group = None;
    }

    pub fn needs_glyph_atlas(&self) -> bool {
        self.atlas_bind_group.is_none()
    }

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

        self.atlas_bind_group = Some(build::binding(
            device,
            "Text Glyph Atlas Bind Group",
            &self.texture_bind_group_layout,
            vec![
                wgpu::BindingResource::TextureView(atlas_view),
                wgpu::BindingResource::Sampler(sampler),
                self.character_color_buffer.as_entire_binding(),
            ],
        ));
    }
}

impl nightshade_wgpu_geometry::backend::OverlaySubPass for TextPass {
    fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        configs: &nightshade_wgpu::backend::RenderInputs,
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
            .query_ref::<&nightshade_text::contract::RenderText>()
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
                        flags: vertex.flags,
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
                padding: [0.0; 2],
            };

            let offset = i * aligned_size;
            queue.write_buffer(
                &self.text_uniform_buffer,
                offset as u64,
                bytemuck::cast_slice(&[text_uniforms]),
            );
        }
    }

    fn wants(&self) -> &'static [&'static str] {
        &["glyph_atlas"]
    }

    fn receive(&mut self, _name: &str, device: &wgpu::Device, view: &wgpu::TextureView) {
        if self.atlas_bind_group.is_none() {
            self.update_glyph_atlas(device, view);
        }
    }

    fn invalidate(&mut self) {
        self.release_glyph_atlas();
    }

    fn record(
        &mut self,
        render_pass: &mut wgpu::RenderPass<'_>,
        device: &wgpu::Device,
        _configs: &nightshade_wgpu::backend::RenderInputs,
    ) {
        TextPass::record(self, render_pass, device);
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
