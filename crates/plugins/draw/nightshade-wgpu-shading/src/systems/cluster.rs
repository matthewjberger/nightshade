pub use nightshade_render::data::config::{
    CLUSTER_GRID_X, CLUSTER_GRID_Y, CLUSTER_GRID_Z, MAX_LIGHTS_PER_CLUSTER, cluster_slice_near,
};
use nightshade_wgpu::backend::PassBuild;
use nightshade_wgpu::backend::build;
use nightshade_wgpu::wgpu;

pub const TOTAL_CLUSTERS: u32 = nightshade_render::data::config::CLUSTER_COUNT;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ClusterUniforms {
    pub inverse_projection: [[f32; 4]; 4],
    pub screen_size: [f32; 2],
    pub z_near: f32,
    pub z_far: f32,
    pub cluster_count: [u32; 4],
    pub tile_size: [f32; 2],
    pub num_lights: u32,
    pub num_directional_lights: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ClusterBounds {
    pub min_point: [f32; 4],
    pub max_point: [f32; 4],
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightGrid {
    pub offset: u32,
    pub count: u32,
}

pub struct ClusterCompute {
    pub bounds_pipeline: wgpu::ComputePipeline,
    pub assign_pipeline: wgpu::ComputePipeline,
    pub bounds_bind_group_layout: wgpu::BindGroupLayout,
    pub assign_bind_group_layout: wgpu::BindGroupLayout,
}

impl ClusterCompute {
    pub fn new(build: PassBuild) -> Self {
        let PassBuild {
            device, composer, ..
        } = build;
        let bounds_shader = composer.compile_wgsl(
            device,
            "Cluster Bounds Shader",
            include_str!("../shaders/cluster_bounds.wgsl"),
        );
        let assign_shader = composer.compile_wgsl(
            device,
            "Cluster Light Assign Shader",
            include_str!("../shaders/cluster_light_assign.wgsl"),
        );

        let storage_entry = |binding: u32, read_only: bool| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };
        let uniform_entry = |binding: u32| build::uniform_at(binding, wgpu::ShaderStages::COMPUTE);

        let bounds_bind_group_layout = build::bind_layout(
            device,
            "Cluster Bounds Bind Group Layout",
            &[uniform_entry(0), storage_entry(1, false)],
        );
        let bounds_pipeline_layout = build::pipeline_layout(
            device,
            "Cluster Bounds Pipeline Layout",
            &[&bounds_bind_group_layout],
        );
        let bounds_pipeline = build::compute(
            device,
            "Cluster Bounds Pipeline",
            &bounds_shader,
            "main",
            &bounds_pipeline_layout,
        );

        let assign_bind_group_layout = build::bind_layout(
            device,
            "Cluster Light Assign Bind Group Layout",
            &[
                uniform_entry(0),
                storage_entry(1, true),
                storage_entry(2, false),
                storage_entry(3, false),
                storage_entry(4, true),
                uniform_entry(5),
            ],
        );
        let assign_pipeline_layout = build::pipeline_layout(
            device,
            "Cluster Light Assign Pipeline Layout",
            &[&assign_bind_group_layout],
        );
        let assign_pipeline = build::compute(
            device,
            "Cluster Light Assign Pipeline",
            &assign_shader,
            "main",
            &assign_pipeline_layout,
        );

        Self {
            bounds_pipeline,
            assign_pipeline,
            bounds_bind_group_layout,
            assign_bind_group_layout,
        }
    }
}
