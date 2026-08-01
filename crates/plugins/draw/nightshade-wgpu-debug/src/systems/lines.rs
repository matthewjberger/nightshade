pub use nightshade_render::components::MAX_LINES;
use nightshade_render::data::bounding_volume::extract_frustum_planes;
use nightshade_wgpu::backend::PassBuild;
use nightshade_wgpu::backend::build;
use nightshade_wgpu::wgpu;
use wgpu::{BindGroup, BindGroupLayout, ComputePipeline, RenderPipeline};

const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
const INITIAL_LINE_CAPACITY: u32 = 1024;
const INITIAL_BV_CAPACITY: u32 = 1024;
const INITIAL_NORMAL_CAPACITY: u32 = 1024;
const BUFFER_GROWTH_FACTOR: f32 = 2.0;
pub const MAX_BOUNDING_VOLUMES: u32 = 100_000;
pub const MAX_NORMALS: u32 = 1_000_000;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DrawIndexedIndirectCommand {
    pub index_count: u32,
    pub instance_count: u32,
    pub first_index: u32,
    pub vertex_offset: i32,
    pub first_instance: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuBoundingVolumeData {
    pub center: [f32; 3],
    pub padding_0: f32,
    pub half_extents: [f32; 3],
    pub padding_1: f32,
    pub orientation: [f32; 4],
    pub transform: [[f32; 4]; 4],
    pub color: [f32; 4],
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BoundingVolumeParams {
    pub bounding_volume_count: u32,
    pub line_offset: u32,
    pub padding: [u32; 2],
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuNormalData {
    pub position: [f32; 3],
    pub padding_0: f32,
    pub normal: [f32; 3],
    pub padding_1: f32,
    pub transform: [[f32; 4]; 4],
    pub color: [f32; 4],
    pub length: f32,
    pub padding_2: [f32; 3],
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct NormalParams {
    pub(crate) normal_count: u32,
    pub line_offset: u32,
    pub padding: [u32; 2],
}

#[repr(C, align(16))]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuLineData {
    pub start: [f32; 4],
    pub end: [f32; 4],
    pub color: [f32; 4],
    pub entity_id: u32,
    pub visible: u32,
    pub depth_mode: u32,
    pub padding: u32,
}

#[repr(C, align(16))]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CullingData {
    pub view_proj: [[f32; 4]; 4],
    pub frustum_planes: [[f32; 4]; 6],
    pub(crate) line_count: u32,
    pub padding: [u32; 15],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LineVertex {
    pub position: [f32; 3],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LineUniform {
    pub view_proj: [[f32; 4]; 4],
}

impl From<&nightshade_render::data::config::RenderLine> for GpuLineData {
    fn from(line: &nightshade_render::data::config::RenderLine) -> Self {
        Self {
            start: [line.start.x, line.start.y, line.start.z, 1.0],
            end: [line.end.x, line.end.y, line.end.z, 1.0],
            color: [line.color.x, line.color.y, line.color.z, line.color.w],
            entity_id: line.entity_id,
            visible: 1,
            depth_mode: line.depth_mode,
            padding: 0,
        }
    }
}

impl From<&nightshade_render::data::config::RenderBoundingVolume> for GpuBoundingVolumeData {
    fn from(volume: &nightshade_render::data::config::RenderBoundingVolume) -> Self {
        Self {
            center: [volume.center.x, volume.center.y, volume.center.z],
            padding_0: 0.0,
            half_extents: [
                volume.half_extents.x,
                volume.half_extents.y,
                volume.half_extents.z,
            ],
            padding_1: 0.0,
            orientation: [
                volume.orientation.x,
                volume.orientation.y,
                volume.orientation.z,
                volume.orientation.w,
            ],
            transform: volume.transform.into(),
            color: [
                volume.color.x,
                volume.color.y,
                volume.color.z,
                volume.color.w,
            ],
        }
    }
}

impl From<&nightshade_render::data::config::RenderNormal> for GpuNormalData {
    fn from(normal: &nightshade_render::data::config::RenderNormal) -> Self {
        Self {
            position: [normal.position.x, normal.position.y, normal.position.z],
            padding_0: 0.0,
            normal: [normal.normal.x, normal.normal.y, normal.normal.z],
            padding_1: 0.0,
            transform: normal.transform.into(),
            color: [
                normal.color.x,
                normal.color.y,
                normal.color.z,
                normal.color.w,
            ],
            length: normal.length,
            padding_2: [0.0, 0.0, 0.0],
        }
    }
}

pub struct LinesPass {
    culling_enabled: bool,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    line_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    storage_bind_group_layout: BindGroupLayout,
    storage_bind_group: BindGroup,
    pipeline: RenderPipeline,
    draw_commands_buffer: wgpu::Buffer,
    draw_count_buffer: wgpu::Buffer,
    culling_bind_group_layout: BindGroupLayout,
    culling_bind_group: BindGroup,
    culling_pipeline: ComputePipeline,
    culling_data_buffer: wgpu::Buffer,
    pub(crate) line_count: u32,
    pub(crate) current_capacity: u32,
    bv_buffer: wgpu::Buffer,
    bv_params_buffer: wgpu::Buffer,
    bv_bind_group_layout: BindGroupLayout,
    bv_bind_group: BindGroup,
    bv_pipeline: ComputePipeline,
    pub(crate) bv_count: u32,
    pub(crate) bv_capacity: u32,
    pub(crate) bv_line_count: u32,
    normal_buffer: wgpu::Buffer,
    normal_params_buffer: wgpu::Buffer,
    normal_bind_group_layout: BindGroupLayout,
    normal_bind_group: BindGroup,
    normal_pipeline: ComputePipeline,
    pub(crate) normal_count: u32,
    pub(crate) normal_capacity: u32,
    pub(crate) normal_line_count: u32,
}

fn main_compute_pipeline(
    device: &wgpu::Device,
    label: &str,
    layout: &wgpu::PipelineLayout,
    module: &wgpu::ShaderModule,
) -> wgpu::ComputePipeline {
    device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some(label),
        layout: Some(layout),
        module,
        entry_point: Some("main"),
        compilation_options: Default::default(),
        cache: None,
    })
}

impl LinesPass {
    pub fn new(build: PassBuild, format: wgpu::TextureFormat) -> Self {
        let PassBuild {
            device, composer, ..
        } = build;
        let vertices = [
            LineVertex {
                position: [0.0, 0.0, 0.0],
            },
            LineVertex {
                position: [1.0, 0.0, 0.0],
            },
        ];

        let vertex_buffer = wgpu::util::DeviceExt::create_buffer_init(
            device,
            &wgpu::util::BufferInitDescriptor {
                label: Some("Line Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            },
        );

        let indices = [0u32, 1u32];
        let index_buffer = wgpu::util::DeviceExt::create_buffer_init(
            device,
            &wgpu::util::BufferInitDescriptor {
                label: Some("Line Index Buffer"),
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX,
            },
        );

        let line_buffer = build::buffer(
            device,
            "Line Buffer",
            (std::mem::size_of::<GpuLineData>() * INITIAL_LINE_CAPACITY as usize) as u64,
            wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
        );

        let uniform_buffer = build::uniform(
            device,
            "Line Uniform Buffer",
            std::mem::size_of::<LineUniform>() as u64,
        );

        let bind_group_layout = build::bind_layout(
            device,
            "Line Bind Group Layout",
            &[build::uniform_at(0, wgpu::ShaderStages::VERTEX)],
        );

        let bind_group = build::binding(
            device,
            "Line Bind Group",
            &bind_group_layout,
            vec![uniform_buffer.as_entire_binding()],
        );

        let storage_bind_group_layout = build::bind_layout(
            device,
            "Line Storage Bind Group Layout",
            &[build::storage_at(0, false, wgpu::ShaderStages::VERTEX)],
        );

        let storage_bind_group = build::binding(
            device,
            "Line Storage Bind Group",
            &storage_bind_group_layout,
            vec![line_buffer.as_entire_binding()],
        );

        let shader = composer.compile_wgsl(
            device,
            "line_gpu.wgsl",
            include_str!("../shaders/line_gpu.wgsl"),
        );

        let pipeline_layout = build::pipeline_layout(
            device,
            "Line Pipeline Layout",
            &[&bind_group_layout, &storage_bind_group_layout],
        );

        let pipeline = build::Pipe::new(device, "Line Pipeline", &shader)
            .layout(&pipeline_layout)
            .buffers(&[wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<LineVertex>() as wgpu::BufferAddress,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &wgpu::vertex_attr_array![0 => Float32x3],
            }])
            .target(wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })
            .depth(wgpu::DepthStencilState {
                format: DEPTH_FORMAT,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::GreaterEqual),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            })
            .lines()
            .vertex("vs_main")
            .fragment("fs_main")
            .make();

        let draw_commands_buffer = build::buffer(
            device,
            "Line Draw Commands Buffer",
            (std::mem::size_of::<DrawIndexedIndirectCommand>() * MAX_LINES as usize) as u64,
            wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::INDIRECT
                | wgpu::BufferUsages::COPY_DST,
        );

        let draw_count_buffer = build::buffer(
            device,
            "Line Draw Count Buffer",
            std::mem::size_of::<u32>() as u64,
            wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::INDIRECT
                | wgpu::BufferUsages::COPY_DST,
        );

        let culling_data_buffer = build::uniform(
            device,
            "Line Culling Data Buffer",
            std::mem::size_of::<CullingData>() as u64,
        );

        let culling_shader = composer.compile_wgsl(
            device,
            "Line Culling Shader",
            include_str!("../shaders/line_culling_gpu.wgsl"),
        );

        let culling_bind_group_layout = build::bind_layout(
            device,
            "Line Culling Bind Group Layout",
            &[
                build::storage_at(0, false, wgpu::ShaderStages::COMPUTE),
                build::storage_at(1, true, wgpu::ShaderStages::COMPUTE),
                build::storage_at(2, true, wgpu::ShaderStages::COMPUTE),
                build::uniform_at(3, wgpu::ShaderStages::COMPUTE),
            ],
        );

        let culling_bind_group = build::binding(
            device,
            "Line Culling Bind Group",
            &culling_bind_group_layout,
            vec![
                line_buffer.as_entire_binding(),
                draw_commands_buffer.as_entire_binding(),
                draw_count_buffer.as_entire_binding(),
                culling_data_buffer.as_entire_binding(),
            ],
        );

        let culling_pipeline_layout = build::pipeline_layout(
            device,
            "Line Culling Pipeline Layout",
            &[&culling_bind_group_layout],
        );

        let culling_pipeline = main_compute_pipeline(
            device,
            "Line Culling Pipeline",
            &culling_pipeline_layout,
            &culling_shader,
        );

        let bv_buffer = build::storage(
            device,
            "Bounding Volume Buffer",
            (std::mem::size_of::<GpuBoundingVolumeData>() * INITIAL_BV_CAPACITY as usize) as u64,
        );

        let bv_params_buffer = build::uniform(
            device,
            "Bounding Volume Params Buffer",
            std::mem::size_of::<BoundingVolumeParams>() as u64,
        );

        let bv_bind_group_layout = build::bind_layout(
            device,
            "Bounding Volume Bind Group Layout",
            &[
                build::storage_at(0, false, wgpu::ShaderStages::COMPUTE),
                build::storage_at(1, true, wgpu::ShaderStages::COMPUTE),
                build::uniform_at(2, wgpu::ShaderStages::COMPUTE),
            ],
        );

        let bv_bind_group = build::binding(
            device,
            "Bounding Volume Bind Group",
            &bv_bind_group_layout,
            vec![
                bv_buffer.as_entire_binding(),
                line_buffer.as_entire_binding(),
                bv_params_buffer.as_entire_binding(),
            ],
        );

        let bv_shader = composer.compile_wgsl(
            device,
            "Bounding Volume Lines Shader",
            include_str!("../shaders/bounding_volume_lines.wgsl"),
        );

        let bv_pipeline_layout = build::pipeline_layout(
            device,
            "Bounding Volume Pipeline Layout",
            &[&bv_bind_group_layout],
        );

        let bv_pipeline = main_compute_pipeline(
            device,
            "Bounding Volume Lines Pipeline",
            &bv_pipeline_layout,
            &bv_shader,
        );

        let normal_buffer = build::storage(
            device,
            "Normal Buffer",
            (std::mem::size_of::<GpuNormalData>() * INITIAL_NORMAL_CAPACITY as usize) as u64,
        );

        let normal_params_buffer = build::uniform(
            device,
            "Normal Params Buffer",
            std::mem::size_of::<NormalParams>() as u64,
        );

        let normal_bind_group_layout = build::bind_layout(
            device,
            "Normal Bind Group Layout",
            &[
                build::storage_at(0, false, wgpu::ShaderStages::COMPUTE),
                build::storage_at(1, true, wgpu::ShaderStages::COMPUTE),
                build::uniform_at(2, wgpu::ShaderStages::COMPUTE),
            ],
        );

        let normal_bind_group = build::binding(
            device,
            "Normal Bind Group",
            &normal_bind_group_layout,
            vec![
                normal_buffer.as_entire_binding(),
                line_buffer.as_entire_binding(),
                normal_params_buffer.as_entire_binding(),
            ],
        );

        let normal_shader = composer.compile_wgsl(
            device,
            "Normal Lines Shader",
            include_str!("../shaders/normal_lines.wgsl"),
        );

        let normal_pipeline_layout = build::pipeline_layout(
            device,
            "Normal Pipeline Layout",
            &[&normal_bind_group_layout],
        );

        let normal_pipeline = main_compute_pipeline(
            device,
            "Normal Lines Pipeline",
            &normal_pipeline_layout,
            &normal_shader,
        );

        Self {
            culling_enabled: false,
            vertex_buffer,
            index_buffer,
            line_buffer,
            uniform_buffer,
            bind_group,
            storage_bind_group_layout,
            storage_bind_group,
            pipeline,
            draw_commands_buffer,
            draw_count_buffer,
            culling_bind_group_layout,
            culling_bind_group,
            culling_pipeline,
            culling_data_buffer,
            line_count: 0,
            current_capacity: INITIAL_LINE_CAPACITY,
            bv_buffer,
            bv_params_buffer,
            bv_bind_group_layout,
            bv_bind_group,
            bv_pipeline,
            bv_count: 0,
            bv_capacity: INITIAL_BV_CAPACITY,
            bv_line_count: 0,
            normal_buffer,
            normal_params_buffer,
            normal_bind_group_layout,
            normal_bind_group,
            normal_pipeline,
            normal_count: 0,
            normal_capacity: INITIAL_NORMAL_CAPACITY,
            normal_line_count: 0,
        }
    }

    fn ensure_capacity(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, required_lines: u32) {
        if required_lines <= self.current_capacity {
            return;
        }

        let new_capacity = (required_lines as f32 * BUFFER_GROWTH_FACTOR).ceil() as u32;
        let new_capacity = new_capacity.min(MAX_LINES);

        tracing::info!(
            "Growing Line buffers from {} to {} lines ({} -> {} bytes)",
            self.current_capacity,
            new_capacity,
            self.current_capacity as u64 * std::mem::size_of::<GpuLineData>() as u64,
            new_capacity as u64 * std::mem::size_of::<GpuLineData>() as u64
        );

        let new_line_buffer_size = std::mem::size_of::<GpuLineData>() * new_capacity as usize;
        let new_line_buffer = build::buffer(
            device,
            "Line Buffer (Resized)",
            new_line_buffer_size as u64,
            wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
        );

        let old_line_data_size = std::mem::size_of::<GpuLineData>() * self.line_count as usize;
        if old_line_data_size > 0 {
            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Line Buffer Resize Copy"),
            });

            encoder.copy_buffer_to_buffer(
                &self.line_buffer,
                0,
                &new_line_buffer,
                0,
                old_line_data_size as u64,
            );

            queue.submit(std::iter::once(encoder.finish()));
        }

        self.line_buffer = new_line_buffer;

        self.storage_bind_group = build::binding(
            device,
            "Line Storage Bind Group",
            &self.storage_bind_group_layout,
            vec![self.line_buffer.as_entire_binding()],
        );

        self.culling_bind_group = build::binding(
            device,
            "Line Culling Bind Group",
            &self.culling_bind_group_layout,
            vec![
                self.line_buffer.as_entire_binding(),
                self.draw_commands_buffer.as_entire_binding(),
                self.draw_count_buffer.as_entire_binding(),
                self.culling_data_buffer.as_entire_binding(),
            ],
        );

        self.bv_bind_group = build::binding(
            device,
            "Bounding Volume Bind Group",
            &self.bv_bind_group_layout,
            vec![
                self.bv_buffer.as_entire_binding(),
                self.line_buffer.as_entire_binding(),
                self.bv_params_buffer.as_entire_binding(),
            ],
        );

        self.normal_bind_group = build::binding(
            device,
            "Normal Bind Group",
            &self.normal_bind_group_layout,
            vec![
                self.normal_buffer.as_entire_binding(),
                self.line_buffer.as_entire_binding(),
                self.normal_params_buffer.as_entire_binding(),
            ],
        );

        self.current_capacity = new_capacity;
    }

    fn ensure_bv_capacity(&mut self, device: &wgpu::Device, required_bvs: u32) {
        if required_bvs <= self.bv_capacity {
            return;
        }

        let new_capacity = (required_bvs as f32 * BUFFER_GROWTH_FACTOR).ceil() as u32;
        let new_capacity = new_capacity.min(MAX_BOUNDING_VOLUMES);

        self.bv_buffer = build::storage(
            device,
            "Bounding Volume Buffer (Resized)",
            (std::mem::size_of::<GpuBoundingVolumeData>() * new_capacity as usize) as u64,
        );

        self.bv_bind_group = build::binding(
            device,
            "Bounding Volume Bind Group",
            &self.bv_bind_group_layout,
            vec![
                self.bv_buffer.as_entire_binding(),
                self.line_buffer.as_entire_binding(),
                self.bv_params_buffer.as_entire_binding(),
            ],
        );

        self.bv_capacity = new_capacity;
    }

    fn update_bounding_volumes(&mut self, queue: &wgpu::Queue, bv_data: &[GpuBoundingVolumeData]) {
        let count = (bv_data.len() as u32).min(self.bv_capacity);
        self.bv_count = count;
        self.bv_line_count = self.bv_count * 12;

        if !bv_data.is_empty() {
            queue.write_buffer(
                &self.bv_buffer,
                0,
                bytemuck::cast_slice(&bv_data[..count as usize]),
            );

            let params = BoundingVolumeParams {
                bounding_volume_count: self.bv_count,
                line_offset: self.line_count,
                padding: [0, 0],
            };
            queue.write_buffer(&self.bv_params_buffer, 0, bytemuck::cast_slice(&[params]));
        }
    }

    pub fn generate_bv_lines(&self, encoder: &mut wgpu::CommandEncoder) {
        if self.bv_count == 0 {
            return;
        }

        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Bounding Volume Lines Pass"),
            timestamp_writes: None,
        });

        compute_pass.set_pipeline(&self.bv_pipeline);
        compute_pass.set_bind_group(0, &self.bv_bind_group, &[]);

        let workgroup_size = 64;
        let dispatch_size = self.bv_count.div_ceil(workgroup_size);
        compute_pass.dispatch_workgroups(dispatch_size, 1, 1);
    }

    fn ensure_normal_capacity(&mut self, device: &wgpu::Device, required_normals: u32) {
        if required_normals <= self.normal_capacity {
            return;
        }

        let new_capacity = (required_normals as f32 * BUFFER_GROWTH_FACTOR).ceil() as u32;
        let new_capacity = new_capacity.min(MAX_NORMALS);

        self.normal_buffer = build::storage(
            device,
            "Normal Buffer (Resized)",
            (std::mem::size_of::<GpuNormalData>() * new_capacity as usize) as u64,
        );

        self.normal_bind_group = build::binding(
            device,
            "Normal Bind Group",
            &self.normal_bind_group_layout,
            vec![
                self.normal_buffer.as_entire_binding(),
                self.line_buffer.as_entire_binding(),
                self.normal_params_buffer.as_entire_binding(),
            ],
        );

        self.normal_capacity = new_capacity;
    }

    fn update_normals(&mut self, queue: &wgpu::Queue, normal_data: &[GpuNormalData]) {
        let count = (normal_data.len() as u32).min(self.normal_capacity);
        self.normal_count = count;
        self.normal_line_count = self.normal_count;

        if !normal_data.is_empty() {
            queue.write_buffer(
                &self.normal_buffer,
                0,
                bytemuck::cast_slice(&normal_data[..count as usize]),
            );

            let params = NormalParams {
                normal_count: self.normal_count,
                line_offset: self.line_count + self.bv_line_count,
                padding: [0, 0],
            };
            queue.write_buffer(
                &self.normal_params_buffer,
                0,
                bytemuck::cast_slice(&[params]),
            );
        }
    }

    pub fn generate_normal_lines(&self, encoder: &mut wgpu::CommandEncoder) {
        if self.normal_count == 0 {
            return;
        }

        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Normal Lines Pass"),
            timestamp_writes: None,
        });

        compute_pass.set_pipeline(&self.normal_pipeline);
        compute_pass.set_bind_group(0, &self.normal_bind_group, &[]);

        let workgroup_size = 256;
        let dispatch_size = self.normal_count.div_ceil(workgroup_size);
        compute_pass.dispatch_workgroups(dispatch_size, 1, 1);
    }

    pub fn total_line_count(&self) -> u32 {
        self.line_count + self.bv_line_count + self.normal_line_count
    }

    pub fn update_uniforms(&self, queue: &wgpu::Queue, view_proj: nalgebra_glm::Mat4) {
        let uniform = LineUniform {
            view_proj: view_proj.into(),
        };
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniform]));
    }

    pub fn update_culling_data(
        &self,
        queue: &wgpu::Queue,
        view_proj: nalgebra_glm::Mat4,
        frustum_planes: [nalgebra_glm::Vec4; 6],
    ) {
        let culling_data = CullingData {
            view_proj: view_proj.into(),
            frustum_planes: frustum_planes.map(|v| [v.x, v.y, v.z, v.w]),
            line_count: self.total_line_count(),
            padding: [0; 15],
        };
        queue.write_buffer(
            &self.culling_data_buffer,
            0,
            bytemuck::cast_slice(&[culling_data]),
        );
    }

    pub fn upload_lines(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        lines: &[nightshade_render::data::config::RenderLine],
    ) {
        let line_data: Vec<GpuLineData> = lines.iter().map(GpuLineData::from).collect();
        let line_count = line_data.len() as u32;
        let total_needed = line_count + self.bv_line_count;
        if total_needed > self.current_capacity {
            self.ensure_capacity(device, queue, total_needed);
        }
        self.update_line_data(queue, &line_data);
    }

    pub fn upload_bounding_volumes(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        volumes: &[nightshade_render::data::config::RenderBoundingVolume],
    ) {
        let bv_data: Vec<GpuBoundingVolumeData> =
            volumes.iter().map(GpuBoundingVolumeData::from).collect();
        let bv_count = bv_data.len() as u32;
        if bv_count > self.bv_capacity {
            self.ensure_bv_capacity(device, bv_count);
        }
        let total_lines_needed = self.line_count + bv_count * 12;
        if total_lines_needed > self.current_capacity {
            self.ensure_capacity(device, queue, total_lines_needed);
        }
        self.update_bounding_volumes(queue, &bv_data);
    }

    pub fn upload_normals(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        normals: &[nightshade_render::data::config::RenderNormal],
    ) {
        let normal_data: Vec<GpuNormalData> = normals.iter().map(GpuNormalData::from).collect();
        let normal_count = normal_data.len() as u32;
        if normal_count > self.normal_capacity {
            self.ensure_normal_capacity(device, normal_count);
        }
        let total_lines_needed = self.line_count + self.bv_line_count + normal_count;
        if total_lines_needed > self.current_capacity {
            self.ensure_capacity(device, queue, total_lines_needed);
        }
        self.update_normals(queue, &normal_data);
    }

    pub fn clear_bounding_volumes(&mut self) {
        self.bv_count = 0;
        self.bv_line_count = 0;
    }

    pub fn clear_normals(&mut self) {
        self.normal_count = 0;
        self.normal_line_count = 0;
    }

    fn update_line_data(&mut self, queue: &wgpu::Queue, line_data: &[GpuLineData]) {
        if !line_data.is_empty() {
            let clamped_count = (line_data.len() as u32).min(self.current_capacity);
            self.line_count = clamped_count;
            queue.write_buffer(
                &self.line_buffer,
                0,
                bytemuck::cast_slice(&line_data[..clamped_count as usize]),
            );
        } else {
            self.line_count = 0;
        }
    }

    pub fn encode_compute(&self, encoder: &mut wgpu::CommandEncoder, culling_enabled: bool) {
        if self.total_line_count() == 0 {
            return;
        }

        self.generate_bv_lines(encoder);
        self.generate_normal_lines(encoder);

        if culling_enabled {
            self.cull_lines(encoder);
        }
    }

    pub fn record(&self, render_pass: &mut wgpu::RenderPass<'_>, culling_enabled: bool) {
        let total_lines = self.total_line_count();
        if total_lines == 0 {
            return;
        }

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_bind_group(1, &self.storage_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

        if culling_enabled {
            if cfg!(target_os = "macos") || cfg!(target_arch = "wasm32") {
                render_pass.multi_draw_indexed_indirect(&self.draw_commands_buffer, 0, total_lines);
            } else {
                render_pass.multi_draw_indexed_indirect_count(
                    &self.draw_commands_buffer,
                    0,
                    &self.draw_count_buffer,
                    0,
                    MAX_LINES,
                );
            }
        } else {
            render_pass.draw_indexed(0..2, 0, 0..total_lines);
        }
    }

    pub fn cull_lines(&self, encoder: &mut wgpu::CommandEncoder) {
        let total = self.total_line_count();
        if total > 0 {
            encoder.clear_buffer(&self.draw_count_buffer, 0, None);

            let commands_size =
                (total as u64) * std::mem::size_of::<DrawIndexedIndirectCommand>() as u64;
            encoder.clear_buffer(&self.draw_commands_buffer, 0, Some(commands_size));

            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Line Culling Pass"),
                timestamp_writes: None,
            });

            compute_pass.set_pipeline(&self.culling_pipeline);
            compute_pass.set_bind_group(0, &self.culling_bind_group, &[]);

            let workgroup_size = 256;
            let dispatch_size = total.div_ceil(workgroup_size);
            let max_dispatch = 65535;
            let final_dispatch = dispatch_size.min(max_dispatch);

            compute_pass.dispatch_workgroups(final_dispatch, 1, 1);
        }
    }
}

impl nightshade_wgpu_geometry::backend::OverlaySubPass for LinesPass {
    fn prepare(
        &mut self,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
        configs: &nightshade_wgpu::backend::RenderInputs,
    ) {
        self.culling_enabled = configs.scene.gpu_culling_enabled;
        if let Some(camera_matrices) = configs.scene.render_view.as_ref() {
            let view_proj = camera_matrices.projection * camera_matrices.view;

            self.update_uniforms(queue, view_proj);

            let frustum_planes = extract_frustum_planes(&view_proj);
            self.update_culling_data(queue, view_proj, frustum_planes);
        }
    }

    fn encode_compute(&mut self, encoder: &mut wgpu::CommandEncoder) {
        LinesPass::encode_compute(self, encoder, self.culling_enabled);
    }

    fn record(
        &mut self,
        render_pass: &mut wgpu::RenderPass<'_>,
        _device: &wgpu::Device,
        _configs: &nightshade_wgpu::backend::RenderInputs,
    ) {
        LinesPass::record(self, render_pass, self.culling_enabled);
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
