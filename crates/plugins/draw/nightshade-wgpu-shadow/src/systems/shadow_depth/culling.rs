use nightshade_wgpu::backend::PassBuild;
use nightshade_wgpu::backend::build;
use nightshade_wgpu::wgpu;
use std::collections::HashMap;

use bytemuck::Zeroable;
use wgpu::util::DeviceExt;

use super::types::{
    MAX_POINT_LIGHT_SHADOWS, MAX_SPOTLIGHT_SHADOWS, NUM_SHADOW_CASCADES, POINT_SHADOW_NUM_FACES,
    ShadowCullUniforms, ShadowDrawIndexedIndirect, ShadowMeshBounds, ShadowMeshGeo, ShadowOccluder,
};

pub(super) const MAX_SHADOW_VIEWS: usize =
    NUM_SHADOW_CASCADES + MAX_SPOTLIGHT_SHADOWS + MAX_POINT_LIGHT_SHADOWS * POINT_SHADOW_NUM_FACES;

const INITIAL_OCCLUDERS: usize = 256;
const INITIAL_MESH_TABLE: usize = 64;
const INITIAL_COMMANDS: usize = MAX_SHADOW_VIEWS * 16;
const INITIAL_VISIBLE: usize = MAX_SHADOW_VIEWS * INITIAL_OCCLUDERS;

pub(super) struct ShadowCulling {
    pub mesh_geo: Vec<ShadowMeshGeo>,
    pub mesh_bounds: Vec<ShadowMeshBounds>,
    pub geo_id_of: HashMap<u64, u32>,
    pub mesh_table_generation: u64,
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
    batch_of_geo: HashMap<u32, u32>,
    batch_geo_ids: Vec<u32>,
    template_dirty: bool,
    bind_groups_dirty: bool,
}

impl ShadowCulling {
    pub fn new(
        build: PassBuild,
        uniform_bind_group_layout: &wgpu::BindGroupLayout,
        point_uniform_bind_group_layout: &wgpu::BindGroupLayout,
        texture_bind_group_layout: &wgpu::BindGroupLayout,
        material_shader_defs: &[(&str, naga_oil::compose::ShaderDefValue)],
        point_shadow_format: wgpu::TextureFormat,
    ) -> Self {
        let PassBuild {
            device, composer, ..
        } = build;
        let cull_shader = composer.compile_wgsl(
            device,
            "shadow_cull.wgsl",
            include_str!("../../shaders/shadow_cull.wgsl"),
        );
        let indirect_shader = composer.compile_wgsl_with_defs(
            device,
            "shadow_depth_indirect.wgsl",
            include_str!("../../shaders/shadow_depth_indirect.wgsl"),
            material_shader_defs,
        );
        let point_indirect_shader = composer.compile_wgsl_with_defs(
            device,
            "point_shadow_depth_indirect.wgsl",
            include_str!("../../shaders/point_shadow_depth_indirect.wgsl"),
            material_shader_defs,
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

        let cull_bind_group_layout = build::bind_layout(
            device,
            "Shadow Cull Bind Group Layout",
            &[
                storage(0, true),
                storage(1, true),
                storage(2, true),
                storage(3, false),
                storage(4, false),
                build::uniform_at(5, wgpu::ShaderStages::COMPUTE),
            ],
        );

        let cull_pipeline_layout = build::pipeline_layout(
            device,
            "Shadow Cull Pipeline Layout",
            &[&cull_bind_group_layout],
        );
        let cull_pipeline = build::compute(
            device,
            "Shadow Cull Pipeline",
            &cull_shader,
            "main",
            &cull_pipeline_layout,
        );

        let instance_storage = |binding: u32, visibility: wgpu::ShaderStages| {
            build::storage_at(binding, false, visibility)
        };
        let instance_bind_group_layout = build::bind_layout(
            device,
            "Shadow Indirect Instance Bind Group Layout",
            &[
                instance_storage(0, wgpu::ShaderStages::VERTEX),
                instance_storage(1, wgpu::ShaderStages::VERTEX),
                instance_storage(2, wgpu::ShaderStages::VERTEX),
                instance_storage(3, wgpu::ShaderStages::FRAGMENT),
            ],
        );

        let indirect_pipeline_layout = build::pipeline_layout(
            device,
            "Shadow Indirect Pipeline Layout",
            &[
                uniform_bind_group_layout,
                &instance_bind_group_layout,
                texture_bind_group_layout,
            ],
        );

        let vertex_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<super::types::ShadowVertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: 12,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: 20,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: 28,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32,
                },
            ],
        };

        let mut indirect_pipe =
            build::Pipe::new(device, "Shadow Indirect Pipeline", &indirect_shader)
                .layout(&indirect_pipeline_layout)
                .buffers(std::slice::from_ref(&vertex_layout))
                .depth(build::depth_state(wgpu::TextureFormat::Depth32Float, true))
                .cull_front();
        if let Some(entry) = super::pass::cutout_fragment_entry("fragment_main") {
            indirect_pipe = indirect_pipe.fragment(entry);
        }
        let indirect_pipeline = indirect_pipe.make();

        let point_targets = [Some(wgpu::ColorTargetState {
            format: point_shadow_format,
            blend: None,
            write_mask: wgpu::ColorWrites::ALL,
        })];

        let point_indirect_pipeline_layout = build::pipeline_layout(
            device,
            "Shadow Point Indirect Pipeline Layout",
            &[
                point_uniform_bind_group_layout,
                &instance_bind_group_layout,
                texture_bind_group_layout,
            ],
        );
        let point_indirect_pipeline = build::Pipe::new(
            device,
            "Shadow Point Indirect Pipeline",
            &point_indirect_shader,
        )
        .layout(&point_indirect_pipeline_layout)
        .buffers(&[vertex_layout])
        .targets(&point_targets)
        .depth(build::depth_state(wgpu::TextureFormat::Depth32Float, true))
        .front_face(wgpu::FrontFace::Cw)
        .cull_front()
        .vertex("vs_main")
        .fragment(super::pass::shadow_color_fragment_entry("fs_main"))
        .make();

        let occluder_buffer = build::storage(
            device,
            "Shadow Occluder Buffer",
            (std::mem::size_of::<ShadowOccluder>() * INITIAL_OCCLUDERS) as u64,
        );
        let mesh_geo_buffer = build::storage(
            device,
            "Shadow Mesh Geo Buffer",
            (std::mem::size_of::<ShadowMeshGeo>() * INITIAL_MESH_TABLE) as u64,
        );
        let mesh_bounds_buffer = build::storage(
            device,
            "Shadow Mesh Bounds Buffer",
            (std::mem::size_of::<ShadowMeshBounds>() * INITIAL_MESH_TABLE) as u64,
        );
        let indirect_buffer = build::buffer(
            device,
            "Shadow Indirect Buffer",
            (std::mem::size_of::<ShadowDrawIndexedIndirect>() * INITIAL_COMMANDS) as u64,
            wgpu::BufferUsages::INDIRECT
                | wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST,
        );
        let indirect_reset_buffer = build::buffer(
            device,
            "Shadow Indirect Reset Buffer",
            (std::mem::size_of::<ShadowDrawIndexedIndirect>() * INITIAL_COMMANDS) as u64,
            wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
        );
        let visible_indices_buffer = build::storage(
            device,
            "Shadow Visible Indices Buffer",
            (std::mem::size_of::<u32>() * INITIAL_VISIBLE) as u64,
        );

        let cull_uniform_buffers = std::array::from_fn(|_| {
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Shadow Cull Uniform Buffer"),
                contents: bytemuck::cast_slice(&[ShadowCullUniforms::zeroed()]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            })
        });

        Self {
            mesh_geo: Vec::new(),
            mesh_table_generation: 0,
            mesh_bounds: Vec::new(),
            geo_id_of: HashMap::new(),
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
            batch_of_geo: HashMap::new(),
            batch_geo_ids: Vec::new(),
            template_dirty: true,
            bind_groups_dirty: true,
            view_count: 0,
        }
    }

    pub fn clear_mesh_table(&mut self) {
        self.mesh_geo.clear();
        self.mesh_bounds.clear();
        self.geo_id_of.clear();
        self.mesh_table_dirty = true;
        self.mesh_table_generation = self.mesh_table_generation.wrapping_add(1);
    }

    pub fn register_mesh(
        &mut self,
        name: &str,
        index_count: u32,
        first_index: u32,
        base_vertex: i32,
        vertices: &[nightshade_render::data::geometry::Vertex],
    ) -> u32 {
        let existing = self
            .geo_id_of
            .get(&nightshade_render::data::generational_registry::registry_name_hash(name))
            .copied();
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
        let geo = ShadowMeshGeo {
            index_count,
            first_index,
            base_vertex,
            padding: 0,
        };
        let bounds = ShadowMeshBounds {
            center,
            radius: radius_sq.sqrt(),
        };
        let id = match existing {
            Some(id) => {
                self.mesh_geo[id as usize] = geo;
                self.mesh_bounds[id as usize] = bounds;
                id
            }
            None => {
                let id = self.mesh_geo.len() as u32;
                self.mesh_geo.push(geo);
                self.mesh_bounds.push(bounds);
                self.geo_id_of.insert(
                    nightshade_render::data::generational_registry::registry_name_hash(name),
                    id,
                );
                id
            }
        };
        self.mesh_table_dirty = true;
        id
    }

    fn batch_for(&mut self, mesh_geo_id: u32) -> u32 {
        if let Some(&batch) = self.batch_of_geo.get(&mesh_geo_id) {
            return batch;
        }
        let batch = self.batch_geo_ids.len() as u32;
        self.batch_geo_ids.push(mesh_geo_id);
        self.batch_of_geo.insert(mesh_geo_id, batch);
        self.template_dirty = true;
        batch
    }

    pub fn prepare_frame(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        occluders: &mut [ShadowOccluder],
        transform_buffer: &wgpu::Buffer,
        materials: &wgpu::Buffer,
        view_count: usize,
        upload: super::types::OccluderUpload,
        bind_groups_dirty: bool,
    ) {
        let view_count = view_count.min(MAX_SHADOW_VIEWS);
        if self.view_count != view_count {
            self.template_dirty = true;
        }
        self.view_count = view_count;
        self.bind_groups_dirty |= bind_groups_dirty;
        if self.mesh_table_dirty {
            self.upload_mesh_table(device, queue);
            self.mesh_table_dirty = false;
        }

        let (mut upload_all, touched) = match upload {
            super::types::OccluderUpload::All => {
                self.batch_of_geo.clear();
                self.batch_geo_ids.clear();
                self.template_dirty = true;
                for occluder in occluders.iter_mut() {
                    occluder.batch_id = self.batch_for(occluder.mesh_geo_id);
                }
                (true, Vec::new())
            }
            super::types::OccluderUpload::Touched { slots, set_changed } => {
                for &slot in &slots {
                    let geo = occluders[slot as usize].mesh_geo_id;
                    occluders[slot as usize].batch_id = self.batch_for(geo);
                }
                self.template_dirty |= set_changed;
                (false, slots)
            }
        };

        let batch_count = self.batch_geo_ids.len();
        let occluder_count = occluders.len();
        if occluder_count != self.occluder_count || batch_count != self.batch_count {
            self.template_dirty = true;
        }
        self.batch_count = batch_count;
        self.occluder_count = occluder_count;

        if occluder_count == 0 {
            return;
        }
        if occluder_count > self.occluder_capacity {
            self.occluder_capacity = (occluder_count * 2).max(INITIAL_OCCLUDERS);
            self.occluder_buffer = build::storage(
                device,
                "Shadow Occluder Buffer (Resized)",
                (std::mem::size_of::<ShadowOccluder>() * self.occluder_capacity) as u64,
            );
            upload_all = true;
            self.bind_groups_dirty = true;
        }

        let command_count = self.view_count * batch_count;
        if command_count > self.indirect_capacity {
            self.indirect_capacity = (command_count * 2).max(INITIAL_COMMANDS);
            self.indirect_buffer = build::buffer(
                device,
                "Shadow Indirect Buffer (Resized)",
                (std::mem::size_of::<ShadowDrawIndexedIndirect>() * self.indirect_capacity) as u64,
                wgpu::BufferUsages::INDIRECT
                    | wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_DST,
            );
            self.indirect_reset_buffer = build::buffer(
                device,
                "Shadow Indirect Reset Buffer (Resized)",
                (std::mem::size_of::<ShadowDrawIndexedIndirect>() * self.indirect_capacity) as u64,
                wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
            );
            self.template_dirty = true;
            self.bind_groups_dirty = true;
        }

        let visible_count = self.view_count * occluder_count;
        if visible_count > self.visible_capacity {
            self.visible_capacity = (visible_count * 2).max(INITIAL_VISIBLE);
            self.visible_indices_buffer = build::storage(
                device,
                "Shadow Visible Indices Buffer (Resized)",
                (std::mem::size_of::<u32>() * self.visible_capacity) as u64,
            );
            self.bind_groups_dirty = true;
        }

        if upload_all {
            queue.write_buffer(&self.occluder_buffer, 0, bytemuck::cast_slice(occluders));
        } else {
            let stride = std::mem::size_of::<ShadowOccluder>() as u64;
            for &slot in &touched {
                queue.write_buffer(
                    &self.occluder_buffer,
                    slot as u64 * stride,
                    bytemuck::bytes_of(&occluders[slot as usize]),
                );
            }
        }

        if self.template_dirty {
            let mut batch_capacity = vec![0u32; batch_count];
            for occluder in occluders.iter() {
                batch_capacity[occluder.batch_id as usize] += 1;
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
                    let geo = &self.mesh_geo[self.batch_geo_ids[batch] as usize];
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
            self.template_dirty = false;
        }

        if self.bind_groups_dirty {
            self.rebuild_bind_groups(device, transform_buffer, materials);
            self.bind_groups_dirty = false;
        }
    }

    fn upload_mesh_table(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        if self.mesh_geo.len() > self.mesh_table_capacity {
            self.bind_groups_dirty = true;
            self.mesh_table_capacity = (self.mesh_geo.len() * 2).max(INITIAL_MESH_TABLE);
            self.mesh_geo_buffer = build::storage(
                device,
                "Shadow Mesh Geo Buffer (Resized)",
                (std::mem::size_of::<ShadowMeshGeo>() * self.mesh_table_capacity) as u64,
            );
            self.mesh_bounds_buffer = build::storage(
                device,
                "Shadow Mesh Bounds Buffer (Resized)",
                (std::mem::size_of::<ShadowMeshBounds>() * self.mesh_table_capacity) as u64,
            );
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

    fn rebuild_bind_groups(
        &mut self,
        device: &wgpu::Device,
        transform_buffer: &wgpu::Buffer,
        materials: &wgpu::Buffer,
    ) {
        self.cull_bind_groups.clear();
        for view in 0..MAX_SHADOW_VIEWS {
            let bind_group = build::binding(
                device,
                "Shadow Cull Bind Group",
                &self.cull_bind_group_layout,
                vec![
                    self.occluder_buffer.as_entire_binding(),
                    transform_buffer.as_entire_binding(),
                    self.mesh_bounds_buffer.as_entire_binding(),
                    self.indirect_buffer.as_entire_binding(),
                    self.visible_indices_buffer.as_entire_binding(),
                    self.cull_uniform_buffers[view].as_entire_binding(),
                ],
            );
            self.cull_bind_groups.push(bind_group);
        }

        self.instance_bind_group = Some(build::binding(
            device,
            "Shadow Indirect Instance Bind Group",
            &self.instance_bind_group_layout,
            vec![
                transform_buffer.as_entire_binding(),
                self.visible_indices_buffer.as_entire_binding(),
                self.occluder_buffer.as_entire_binding(),
                materials.as_entire_binding(),
            ],
        ));
    }

    pub fn set_view_frustum(
        &self,
        queue: &wgpu::Queue,
        view: usize,
        frustum_planes: [nalgebra_glm::Vec4; 6],
    ) {
        if view >= MAX_SHADOW_VIEWS {
            return;
        }
        let uniforms = ShadowCullUniforms {
            frustum_planes: frustum_planes.map(|plane| [plane.x, plane.y, plane.z, plane.w]),
            occluder_count: self.occluder_count as u32,
            indirect_offset: (view * self.batch_count) as u32,
            padding_0: 0,
            padding_1: 0,
        };
        queue.write_buffer(
            &self.cull_uniform_buffers[view],
            0,
            bytemuck::cast_slice(&[uniforms]),
        );
    }

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
