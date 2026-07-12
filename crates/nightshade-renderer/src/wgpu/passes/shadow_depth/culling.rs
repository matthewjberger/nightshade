use std::collections::HashMap;

use bytemuck::Zeroable;
use wgpu::util::DeviceExt;

use super::types::{
    MAX_POINT_LIGHT_SHADOWS, MAX_SPOTLIGHT_SHADOWS, NUM_SHADOW_CASCADES, POINT_SHADOW_NUM_FACES,
    ShadowCullUniforms, ShadowDrawIndexedIndirect, ShadowMeshBounds, ShadowMeshGeo, ShadowOccluder,
};

/// Maximum shadow views culled in one frame: the directional cascades plus one
/// per spotlight plus six faces per point light. Buffers are sized for this; the
/// actual per-frame view count is passed to `prepare_frame`.
pub(super) const MAX_SHADOW_VIEWS: usize =
    NUM_SHADOW_CASCADES + MAX_SPOTLIGHT_SHADOWS + MAX_POINT_LIGHT_SHADOWS * POINT_SHADOW_NUM_FACES;

const INITIAL_OCCLUDERS: usize = 256;
const INITIAL_MESH_TABLE: usize = 64;
const INITIAL_COMMANDS: usize = MAX_SHADOW_VIEWS * 16;
const INITIAL_VISIBLE: usize = MAX_SHADOW_VIEWS * INITIAL_OCCLUDERS;

/// GPU-driven directional cascade shadows: a per-occluder object buffer plus
/// per-mesh geometry and bounds tables, a frustum-cull compute pass run once
/// per cascade that scatters surviving occluders into per-mesh indirect
/// commands, and an indirect draw per cascade. Replaces the per-occluder
/// `draw_indexed` CPU loop and adds per-cascade frustum culling the CPU path
/// never had.
pub(super) struct ShadowCulling {
    pub mesh_geo: Vec<ShadowMeshGeo>,
    pub mesh_bounds: Vec<ShadowMeshBounds>,
    pub name_to_geo_id: HashMap<String, u32>,
    mesh_table_dirty: bool,

    occluder_buffer: wgpu::Buffer,
    occluder_capacity: usize,
    mesh_geo_buffer: wgpu::Buffer,
    mesh_bounds_buffer: wgpu::Buffer,
    mesh_table_capacity: usize,

    indirect_buffer: wgpu::Buffer,
    indirect_reset_buffer: wgpu::Buffer,
    indirect_capacity: usize,
    visible_indices_buffer: wgpu::Buffer,
    visible_capacity: usize,

    cull_pipeline: wgpu::ComputePipeline,
    cull_bind_group_layout: wgpu::BindGroupLayout,
    cull_uniform_buffers: [wgpu::Buffer; MAX_SHADOW_VIEWS],
    cull_bind_groups: Vec<wgpu::BindGroup>,

    pub indirect_pipeline: wgpu::RenderPipeline,
    pub point_indirect_pipeline: wgpu::RenderPipeline,
    instance_bind_group_layout: wgpu::BindGroupLayout,
    instance_bind_group: Option<wgpu::BindGroup>,

    batch_count: usize,
    occluder_count: usize,
    view_count: usize,
}

impl ShadowCulling {
    pub fn new(
        device: &wgpu::Device,
        uniform_bind_group_layout: &wgpu::BindGroupLayout,
        point_uniform_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let cull_shader = crate::wgpu::shader_compose::compile_wgsl(
            device,
            "shadow_cull.wgsl",
            include_str!("../../shaders/shadow_cull.wgsl"),
        );
        let indirect_shader = crate::wgpu::shader_compose::compile_wgsl(
            device,
            "shadow_depth_indirect.wgsl",
            include_str!("../../shaders/shadow_depth_indirect.wgsl"),
        );
        let point_indirect_shader = crate::wgpu::shader_compose::compile_wgsl(
            device,
            "point_shadow_depth_indirect.wgsl",
            include_str!("../../shaders/point_shadow_depth_indirect.wgsl"),
        );

        let storage = |binding: u32, read_only: bool| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };

        let cull_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Shadow Cull Bind Group Layout"),
                entries: &[
                    storage(0, true),
                    storage(1, true),
                    storage(2, true),
                    storage(3, false),
                    storage(4, false),
                    wgpu::BindGroupLayoutEntry {
                        binding: 5,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        let cull_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Shadow Cull Pipeline Layout"),
            bind_group_layouts: &[Some(&cull_bind_group_layout)],
            immediate_size: 0,
        });
        let cull_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Shadow Cull Pipeline"),
            layout: Some(&cull_pipeline_layout),
            module: &cull_shader,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });

        let instance_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Shadow Indirect Instance Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        let indirect_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Shadow Indirect Pipeline Layout"),
                bind_group_layouts: &[
                    Some(uniform_bind_group_layout),
                    Some(&instance_bind_group_layout),
                ],
                immediate_size: 0,
            });

        let vertex_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<crate::geometry::Vertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: 12,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: 24,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        };

        let indirect_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Shadow Indirect Pipeline"),
            layout: Some(&indirect_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &indirect_shader,
                entry_point: Some("vertex_main"),
                buffers: std::slice::from_ref(&vertex_layout),
                compilation_options: Default::default(),
            },
            fragment: None,
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::GreaterEqual),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        let point_indirect_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Shadow Point Indirect Pipeline Layout"),
                bind_group_layouts: &[
                    Some(point_uniform_bind_group_layout),
                    Some(&instance_bind_group_layout),
                ],
                immediate_size: 0,
            });
        let point_indirect_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Shadow Point Indirect Pipeline"),
                layout: Some(&point_indirect_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &point_indirect_shader,
                    entry_point: Some("vs_main"),
                    buffers: &[vertex_layout],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &point_indirect_shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::R32Float,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Cw,
                    cull_mode: Some(wgpu::Face::Back),
                    unclipped_depth: false,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    conservative: false,
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_write_enabled: Some(true),
                    depth_compare: Some(wgpu::CompareFunction::GreaterEqual),
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState::default(),
                multiview_mask: None,
                cache: None,
            });

        let occluder_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Shadow Occluder Buffer"),
            size: (std::mem::size_of::<ShadowOccluder>() * INITIAL_OCCLUDERS) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mesh_geo_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Shadow Mesh Geo Buffer"),
            size: (std::mem::size_of::<ShadowMeshGeo>() * INITIAL_MESH_TABLE) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mesh_bounds_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Shadow Mesh Bounds Buffer"),
            size: (std::mem::size_of::<ShadowMeshBounds>() * INITIAL_MESH_TABLE) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let indirect_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Shadow Indirect Buffer"),
            size: (std::mem::size_of::<ShadowDrawIndexedIndirect>() * INITIAL_COMMANDS) as u64,
            usage: wgpu::BufferUsages::INDIRECT
                | wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let indirect_reset_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Shadow Indirect Reset Buffer"),
            size: (std::mem::size_of::<ShadowDrawIndexedIndirect>() * INITIAL_COMMANDS) as u64,
            usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let visible_indices_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Shadow Visible Indices Buffer"),
            size: (std::mem::size_of::<u32>() * INITIAL_VISIBLE) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let cull_uniform_buffers = std::array::from_fn(|_| {
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Shadow Cull Uniform Buffer"),
                contents: bytemuck::cast_slice(&[ShadowCullUniforms::zeroed()]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            })
        });

        Self {
            mesh_geo: Vec::new(),
            mesh_bounds: Vec::new(),
            name_to_geo_id: HashMap::new(),
            mesh_table_dirty: false,
            occluder_buffer,
            occluder_capacity: INITIAL_OCCLUDERS,
            mesh_geo_buffer,
            mesh_bounds_buffer,
            mesh_table_capacity: INITIAL_MESH_TABLE,
            indirect_buffer,
            indirect_reset_buffer,
            indirect_capacity: INITIAL_COMMANDS,
            visible_indices_buffer,
            visible_capacity: INITIAL_VISIBLE,
            cull_pipeline,
            cull_bind_group_layout,
            cull_uniform_buffers,
            cull_bind_groups: Vec::new(),
            indirect_pipeline,
            point_indirect_pipeline,
            instance_bind_group_layout,
            instance_bind_group: None,
            batch_count: 0,
            occluder_count: 0,
            view_count: 0,
        }
    }

    /// Drops every registered mesh so the geo table can be rebuilt from scratch.
    /// Geo ids are reassigned on the next `register_mesh`, so callers must
    /// re-register all live meshes before constructing occluders for the frame.
    pub fn clear_mesh_table(&mut self) {
        self.mesh_geo.clear();
        self.mesh_bounds.clear();
        self.name_to_geo_id.clear();
        self.mesh_table_dirty = true;
    }

    /// Registers a mesh's geometry and computes its bounding sphere, assigning a
    /// stable geo id. Returns the geo id for occluder construction.
    pub fn register_mesh(
        &mut self,
        name: &str,
        index_count: u32,
        first_index: u32,
        base_vertex: i32,
        vertices: &[crate::geometry::Vertex],
    ) -> u32 {
        if let Some(&id) = self.name_to_geo_id.get(name) {
            return id;
        }
        let mut min = [f32::MAX; 3];
        let mut max = [f32::MIN; 3];
        for vertex in vertices {
            for axis in 0..3 {
                min[axis] = min[axis].min(vertex.position[axis]);
                max[axis] = max[axis].max(vertex.position[axis]);
            }
        }
        let center = [
            (min[0] + max[0]) * 0.5,
            (min[1] + max[1]) * 0.5,
            (min[2] + max[2]) * 0.5,
        ];
        let mut radius_sq = 0.0f32;
        for vertex in vertices {
            let dx = vertex.position[0] - center[0];
            let dy = vertex.position[1] - center[1];
            let dz = vertex.position[2] - center[2];
            radius_sq = radius_sq.max(dx * dx + dy * dy + dz * dz);
        }
        let id = self.mesh_geo.len() as u32;
        self.mesh_geo.push(ShadowMeshGeo {
            index_count,
            first_index,
            base_vertex,
            _pad: 0,
        });
        self.mesh_bounds.push(ShadowMeshBounds {
            center,
            radius: radius_sq.sqrt(),
        });
        self.name_to_geo_id.insert(name.to_string(), id);
        self.mesh_table_dirty = true;
        id
    }

    /// Uploads the per-frame occluder set and rebuilds the per-cascade indirect
    /// command template (one command per distinct occluder mesh, per cascade)
    /// plus the matching reset buffer. Resizes GPU buffers and rebuilds bind
    /// groups as needed. `transform_buffer` is the shadow pass transform buffer
    /// that occluder `transform_index` values address.
    pub fn prepare_frame(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        occluders: &mut [ShadowOccluder],
        transform_buffer: &wgpu::Buffer,
        view_count: usize,
        upload_occluders: bool,
    ) {
        self.view_count = view_count.min(MAX_SHADOW_VIEWS);
        if self.mesh_table_dirty {
            self.upload_mesh_table(device, queue);
            self.mesh_table_dirty = false;
        }

        let mut batch_of_geo: HashMap<u32, u32> = HashMap::new();
        let mut batch_geo_ids: Vec<u32> = Vec::new();
        let mut batch_capacity: Vec<u32> = Vec::new();
        for occluder in occluders.iter_mut() {
            let batch_id = *batch_of_geo.entry(occluder.mesh_geo_id).or_insert_with(|| {
                let id = batch_geo_ids.len() as u32;
                batch_geo_ids.push(occluder.mesh_geo_id);
                batch_capacity.push(0);
                id
            });
            occluder.batch_id = batch_id;
            batch_capacity[batch_id as usize] += 1;
        }

        let batch_count = batch_geo_ids.len();
        let occluder_count = occluders.len();
        self.batch_count = batch_count;
        self.occluder_count = occluder_count;

        if occluder_count == 0 {
            return;
        }
        if occluder_count > self.occluder_capacity {
            self.occluder_capacity = (occluder_count * 2).max(INITIAL_OCCLUDERS);
            self.occluder_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Shadow Occluder Buffer (Resized)"),
                size: (std::mem::size_of::<ShadowOccluder>() * self.occluder_capacity) as u64,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }

        let command_count = self.view_count * batch_count;
        if command_count > self.indirect_capacity {
            self.indirect_capacity = (command_count * 2).max(INITIAL_COMMANDS);
            self.indirect_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Shadow Indirect Buffer (Resized)"),
                size: (std::mem::size_of::<ShadowDrawIndexedIndirect>() * self.indirect_capacity)
                    as u64,
                usage: wgpu::BufferUsages::INDIRECT
                    | wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.indirect_reset_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Shadow Indirect Reset Buffer (Resized)"),
                size: (std::mem::size_of::<ShadowDrawIndexedIndirect>() * self.indirect_capacity)
                    as u64,
                usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }

        let visible_count = self.view_count * occluder_count;
        if visible_count > self.visible_capacity {
            self.visible_capacity = (visible_count * 2).max(INITIAL_VISIBLE);
            self.visible_indices_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Shadow Visible Indices Buffer (Resized)"),
                size: (std::mem::size_of::<u32>() * self.visible_capacity) as u64,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }

        if upload_occluders {
            queue.write_buffer(&self.occluder_buffer, 0, bytemuck::cast_slice(occluders));
        }

        let mut batch_prefix = vec![0u32; batch_count];
        let mut running = 0u32;
        for batch in 0..batch_count {
            batch_prefix[batch] = running;
            running += batch_capacity[batch];
        }

        let mut template = vec![ShadowDrawIndexedIndirect::zeroed(); command_count];
        for view in 0..self.view_count {
            let visible_base = (view * occluder_count) as u32;
            for batch in 0..batch_count {
                let geo = &self.mesh_geo[batch_geo_ids[batch] as usize];
                template[view * batch_count + batch] = ShadowDrawIndexedIndirect {
                    index_count: geo.index_count,
                    instance_count: 0,
                    first_index: geo.first_index,
                    base_vertex: geo.base_vertex,
                    first_instance: visible_base + batch_prefix[batch],
                };
            }
        }
        queue.write_buffer(
            &self.indirect_reset_buffer,
            0,
            bytemuck::cast_slice(&template),
        );

        self.rebuild_bind_groups(device, transform_buffer);
    }

    fn upload_mesh_table(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        if self.mesh_geo.len() > self.mesh_table_capacity {
            self.mesh_table_capacity = (self.mesh_geo.len() * 2).max(INITIAL_MESH_TABLE);
            self.mesh_geo_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Shadow Mesh Geo Buffer (Resized)"),
                size: (std::mem::size_of::<ShadowMeshGeo>() * self.mesh_table_capacity) as u64,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.mesh_bounds_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Shadow Mesh Bounds Buffer (Resized)"),
                size: (std::mem::size_of::<ShadowMeshBounds>() * self.mesh_table_capacity) as u64,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }
        if !self.mesh_geo.is_empty() {
            queue.write_buffer(
                &self.mesh_geo_buffer,
                0,
                bytemuck::cast_slice(&self.mesh_geo),
            );
            queue.write_buffer(
                &self.mesh_bounds_buffer,
                0,
                bytemuck::cast_slice(&self.mesh_bounds),
            );
        }
    }

    fn rebuild_bind_groups(&mut self, device: &wgpu::Device, transform_buffer: &wgpu::Buffer) {
        self.cull_bind_groups.clear();
        for view in 0..MAX_SHADOW_VIEWS {
            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Shadow Cull Bind Group"),
                layout: &self.cull_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: self.occluder_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: transform_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: self.mesh_bounds_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: self.indirect_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: self.visible_indices_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 5,
                        resource: self.cull_uniform_buffers[view].as_entire_binding(),
                    },
                ],
            });
            self.cull_bind_groups.push(bind_group);
        }

        self.instance_bind_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Shadow Indirect Instance Bind Group"),
            layout: &self.instance_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: transform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.visible_indices_buffer.as_entire_binding(),
                },
            ],
        }));
    }

    pub fn set_view_frustum(
        &self,
        queue: &wgpu::Queue,
        view: usize,
        frustum_planes: [[f32; 4]; 6],
    ) {
        if view >= MAX_SHADOW_VIEWS {
            return;
        }
        let uniforms = ShadowCullUniforms {
            frustum_planes,
            occluder_count: self.occluder_count as u32,
            indirect_offset: (view * self.batch_count) as u32,
            _pad0: 0,
            _pad1: 0,
        };
        queue.write_buffer(
            &self.cull_uniform_buffers[view],
            0,
            bytemuck::cast_slice(&[uniforms]),
        );
    }

    /// Resets the indirect commands then runs the cull compute for every dirty
    /// view. Clean (cached) views are skipped: their draws are skipped too, so
    /// their reset (zero-instance) commands are never consumed.
    pub fn dispatch_cull(&self, encoder: &mut wgpu::CommandEncoder, view_dirty: &[bool]) {
        if self.occluder_count == 0 || self.batch_count == 0 || self.view_count == 0 {
            return;
        }
        let command_count = self.view_count * self.batch_count;
        encoder.copy_buffer_to_buffer(
            &self.indirect_reset_buffer,
            0,
            &self.indirect_buffer,
            0,
            (std::mem::size_of::<ShadowDrawIndexedIndirect>() * command_count) as u64,
        );

        let dispatch = (self.occluder_count as u32).div_ceil(64);
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Shadow Cull Pass"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&self.cull_pipeline);
        for view in 0..self.view_count {
            if view_dirty.get(view).is_some_and(|dirty| !dirty) {
                continue;
            }
            pass.set_bind_group(0, &self.cull_bind_groups[view], &[]);
            pass.dispatch_workgroups(dispatch, 1, 1);
        }
    }

    /// Issues the indirect draw for one shadow view. The caller must have set the
    /// view's uniform bind group (group 0), viewport, and render pass.
    pub fn draw_view<'a>(
        &'a self,
        pass: &mut wgpu::RenderPass<'a>,
        view: usize,
        supports_multi_draw: bool,
    ) {
        if self.occluder_count == 0 || self.batch_count == 0 {
            return;
        }
        let Some(instance_bind_group) = self.instance_bind_group.as_ref() else {
            return;
        };
        pass.set_bind_group(1, instance_bind_group, &[]);
        let stride = std::mem::size_of::<ShadowDrawIndexedIndirect>() as u64;
        let base = (view * self.batch_count) as u64;
        if supports_multi_draw {
            pass.multi_draw_indexed_indirect(
                &self.indirect_buffer,
                base * stride,
                self.batch_count as u32,
            );
        } else {
            for batch in 0..self.batch_count as u64 {
                pass.draw_indexed_indirect(&self.indirect_buffer, (base + batch) * stride);
            }
        }
    }
}
