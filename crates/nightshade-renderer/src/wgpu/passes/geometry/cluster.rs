//! Shared clustered-lighting GPU types and grid dimensions, used by both the
//! static and skinned mesh passes.

/// Cluster grid tile count along X.
pub const CLUSTER_GRID_X: u32 = 16;
/// Cluster grid tile count along Y.
pub const CLUSTER_GRID_Y: u32 = 9;
/// Cluster grid depth-slice count along Z.
pub const CLUSTER_GRID_Z: u32 = 24;
/// Maximum lights recorded per cluster.
pub const MAX_LIGHTS_PER_CLUSTER: u32 = 256;
/// Total froxel count across the grid.
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

/// GPU uniforms driving cluster bounds and light assignment.
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ClusterUniforms {
    /// Inverse projection matrix, clip to view space.
    pub inverse_projection: [[f32; 4]; 4],
    /// Render target size in pixels.
    pub screen_size: [f32; 2],
    /// Near distance the slices start from.
    pub z_near: f32,
    /// Far distance the slices reach.
    pub z_far: f32,
    /// Grid dimensions X, Y, Z in the first three lanes.
    pub cluster_count: [u32; 4],
    /// Screen tile size in pixels.
    pub tile_size: [f32; 2],
    /// Number of local lights.
    pub num_lights: u32,
    /// Number of directional lights.
    pub num_directional_lights: u32,
}

/// View-space axis-aligned bounds of one cluster.
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ClusterBounds {
    /// Minimum corner in view space.
    pub min_point: [f32; 4],
    /// Maximum corner in view space.
    pub max_point: [f32; 4],
}

/// Per-cluster slice into the light index list.
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightGrid {
    /// Start index into the light index list.
    pub offset: u32,
    /// Number of lights in the cluster.
    pub count: u32,
}

/// The two clustered-lighting compute pipelines (cluster bounds and light
/// assignment) plus their bind group layouts, shared by the static and skinned
/// mesh passes so both build the cluster light grid from one implementation.
pub struct ClusterCompute {
    /// Compute pipeline building per-cluster bounds.
    pub bounds_pipeline: wgpu::ComputePipeline,
    /// Compute pipeline assigning lights to clusters.
    pub assign_pipeline: wgpu::ComputePipeline,
    /// Bind group layout for the bounds pass.
    pub bounds_bind_group_layout: wgpu::BindGroupLayout,
    /// Bind group layout for the assignment pass.
    pub assign_bind_group_layout: wgpu::BindGroupLayout,
}

impl ClusterCompute {
    /// Compiles both cluster compute pipelines and their bind group layouts.
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
