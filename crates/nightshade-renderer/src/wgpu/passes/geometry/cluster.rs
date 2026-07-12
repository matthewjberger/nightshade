//! Shared clustered-lighting GPU types and grid dimensions, used by both the
//! static and skinned mesh passes.

pub const CLUSTER_GRID_X: u32 = 16;
pub const CLUSTER_GRID_Y: u32 = 9;
pub const CLUSTER_GRID_Z: u32 = 24;
pub const MAX_LIGHTS_PER_CLUSTER: u32 = 256;
pub const TOTAL_CLUSTERS: u32 = CLUSTER_GRID_X * CLUSTER_GRID_Y * CLUSTER_GRID_Z;

/// Distance from the camera at which the logarithmic depth slices begin,
/// decoupled from the camera near plane. Cameras commonly set a near plane
/// around 0.1 so the depth buffer keeps precision, but distributing the froxel
/// slices from there spends roughly half of them on the empty space directly in
/// front of the camera and crushes the visible scene into a handful of far
/// slices, which overloads those froxels. Starting the slices a short distance
/// out spreads them across the depth range geometry actually occupies.
const CLUSTER_SLICE_NEAR: f32 = 0.5;

/// Near distance the clustered depth slices are distributed from for a camera
/// with the given near plane. Both the bounds and assignment compute passes and
/// the fragment lookup read this same value, so the grid that is built and the
/// cluster a fragment samples stay consistent. See `CLUSTER_SLICE_NEAR`.
pub fn cluster_slice_near(camera_z_near: f32) -> f32 {
    camera_z_near.max(CLUSTER_SLICE_NEAR)
}

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

/// The two clustered-lighting compute pipelines (cluster bounds and light
/// assignment) plus their bind group layouts, shared by the static and skinned
/// mesh passes so both build the cluster light grid from one implementation.
pub struct ClusterCompute {
    pub bounds_pipeline: wgpu::ComputePipeline,
    pub assign_pipeline: wgpu::ComputePipeline,
    pub bounds_bind_group_layout: wgpu::BindGroupLayout,
    pub assign_bind_group_layout: wgpu::BindGroupLayout,
}

impl ClusterCompute {
    pub fn new(device: &wgpu::Device) -> Self {
        let bounds_shader = crate::wgpu::shader_compose::compile_wgsl(
            device,
            "Cluster Bounds Shader",
            include_str!("../../shaders/cluster_bounds.wgsl"),
        );
        let assign_shader = crate::wgpu::shader_compose::compile_wgsl(
            device,
            "Cluster Light Assign Shader",
            include_str!("../../shaders/cluster_light_assign.wgsl"),
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
        let uniform_entry = |binding: u32| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };

        let bounds_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Cluster Bounds Bind Group Layout"),
                entries: &[uniform_entry(0), storage_entry(1, false)],
            });
        let bounds_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Cluster Bounds Pipeline Layout"),
                bind_group_layouts: &[Some(&bounds_bind_group_layout)],
                immediate_size: 0,
            });
        let bounds_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Cluster Bounds Pipeline"),
            layout: Some(&bounds_pipeline_layout),
            module: &bounds_shader,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });

        let assign_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Cluster Light Assign Bind Group Layout"),
                entries: &[
                    uniform_entry(0),
                    storage_entry(1, true),
                    storage_entry(2, false),
                    storage_entry(3, false),
                    storage_entry(4, true),
                    uniform_entry(5),
                ],
            });
        let assign_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Cluster Light Assign Pipeline Layout"),
                bind_group_layouts: &[Some(&assign_bind_group_layout)],
                immediate_size: 0,
            });
        let assign_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Cluster Light Assign Pipeline"),
            layout: Some(&assign_pipeline_layout),
            module: &assign_shader,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });

        Self {
            bounds_pipeline,
            assign_pipeline,
            bounds_bind_group_layout,
            assign_bind_group_layout,
        }
    }
}
