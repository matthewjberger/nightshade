use wgpu::util::DeviceExt;

use super::types::{
    AreaLightData, AreaUniforms, ClusterUniforms, LightData, LightGrid, MAX_AREA_LIGHTS,
    MAX_LIGHTS_PER_CLUSTER, MaterialData, SkinnedCullObject, SkinnedCullUniforms,
    SkinnedDrawIndexedIndirect, SkinnedObjectData, TOTAL_CLUSTERS,
};

const INITIAL_LIGHTS: usize = 32;
const INITIAL_OBJECTS: usize = 100;
const INITIAL_COMMANDS: usize = 64;

const INITIAL_MATERIALS: usize = 100;

pub(super) struct SkinnedWorldGpuBuffers {
    pub materials_buffer: wgpu::Buffer,
    pub materials_buffer_size: usize,
    pub lights_buffer: wgpu::Buffer,
    pub lights_buffer_size: usize,
    pub object_buffer: wgpu::Buffer,
    pub object_buffer_size: usize,
    pub custom_data_buffer: wgpu::Buffer,
    pub custom_data_buffer_size: usize,
    pub cluster_uniforms_buffer: wgpu::Buffer,
    pub light_grid_buffer: wgpu::Buffer,
    pub light_grid_reset_buffer: wgpu::Buffer,
    pub light_indices_buffer: wgpu::Buffer,
    pub cull_objects_buffer: wgpu::Buffer,
    pub cull_objects_buffer_size: usize,
    pub indirect_commands_buffer: wgpu::Buffer,
    pub indirect_commands_buffer_size: usize,
    pub visible_indices_buffer: wgpu::Buffer,
    pub visible_indices_buffer_size: usize,
    pub cull_uniforms_buffer: wgpu::Buffer,
    pub instance_bind_group: wgpu::BindGroup,
    pub cull_bind_group: wgpu::BindGroup,
    pub cluster_bounds_bind_group: wgpu::BindGroup,
    pub cluster_assign_bind_group: Option<wgpu::BindGroup>,
    pub area_lights_buffer: wgpu::Buffer,
    pub area_uniforms_buffer: wgpu::Buffer,
    pub dense_capacity_buffer: wgpu::Buffer,
    pub dense_capacity_size: usize,
    pub command_map_buffer: wgpu::Buffer,
    pub command_map_size: usize,
    pub batch_meta_buffer: wgpu::Buffer,
    pub batch_bind_group: Option<wgpu::BindGroup>,
}

impl SkinnedWorldGpuBuffers {
    pub fn new(
        device: &wgpu::Device,
        instance_bind_group_layout: &wgpu::BindGroupLayout,
        cull_bind_group_layout: &wgpu::BindGroupLayout,
        cluster_bounds_bind_group_layout: &wgpu::BindGroupLayout,
        cluster_bounds_buffer: &wgpu::Buffer,
        joint_buffers: (&wgpu::Buffer, &wgpu::Buffer),
        morph_displacement_buffer: &wgpu::Buffer,
    ) -> Self {
        let (joint_matrices_buffer, prev_joint_matrices_buffer) = joint_buffers;
        let cull_objects_buffer_size = INITIAL_OBJECTS;
        let cull_objects_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Skinned Mesh Cull Objects Buffer (Per-World)"),
            size: (std::mem::size_of::<SkinnedCullObject>() * cull_objects_buffer_size) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let indirect_commands_buffer_size = INITIAL_COMMANDS;
        let indirect_commands_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Skinned Mesh Indirect Commands Buffer (Per-World)"),
            size: (std::mem::size_of::<SkinnedDrawIndexedIndirect>()
                * indirect_commands_buffer_size) as u64,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::INDIRECT
                | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let visible_indices_buffer_size = INITIAL_OBJECTS;
        let visible_indices_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Skinned Mesh Visible Indices Buffer (Per-World)"),
            size: (std::mem::size_of::<u32>() * visible_indices_buffer_size) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let cull_uniforms_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Skinned Mesh Cull Uniforms Buffer (Per-World)"),
            size: std::mem::size_of::<SkinnedCullUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let materials_buffer_size = INITIAL_MATERIALS;
        let materials_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Skinned Mesh Materials Buffer (Per-World)"),
            size: (std::mem::size_of::<MaterialData>() * materials_buffer_size) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let lights_buffer_size = INITIAL_LIGHTS;
        let lights_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Skinned Mesh Lights Buffer (Per-World)"),
            size: (std::mem::size_of::<LightData>() * lights_buffer_size) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let object_buffer_size = INITIAL_OBJECTS;
        let object_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Skinned Mesh Object Buffer (Per-World)"),
            size: (std::mem::size_of::<SkinnedObjectData>() * object_buffer_size) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let custom_data_buffer_size = INITIAL_OBJECTS;
        let custom_data_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Skinned Mesh Custom Data Buffer (Per-World)"),
            size: (std::mem::size_of::<[f32; 4]>() * custom_data_buffer_size) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let cluster_uniforms_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Skinned Cluster Uniforms Buffer (Per-World)"),
            size: std::mem::size_of::<ClusterUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let light_grid_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Skinned Light Grid Buffer (Per-World)"),
            size: (std::mem::size_of::<LightGrid>() * TOTAL_CLUSTERS as usize) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let light_grid_reset_data: Vec<LightGrid> = vec![
            LightGrid {
                offset: 0,
                count: 0
            };
            TOTAL_CLUSTERS as usize
        ];
        let light_grid_reset_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Skinned Light Grid Reset Buffer (Per-World)"),
                contents: bytemuck::cast_slice(&light_grid_reset_data),
                usage: wgpu::BufferUsages::COPY_SRC,
            });

        let light_indices_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Skinned Light Indices Buffer (Per-World)"),
            size: (std::mem::size_of::<u32>()
                * TOTAL_CLUSTERS as usize
                * MAX_LIGHTS_PER_CLUSTER as usize) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let area_lights_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Skinned Area Lights Buffer (Per-World)"),
            size: (std::mem::size_of::<AreaLightData>() * MAX_AREA_LIGHTS) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let area_uniforms_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Skinned Area Uniforms Buffer (Per-World)"),
            size: std::mem::size_of::<AreaUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let instance_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Skinned Mesh Instance Bind Group (Per-World)"),
            layout: instance_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: materials_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: object_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: lights_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: joint_matrices_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: custom_data_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: morph_displacement_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: visible_indices_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: light_grid_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 8,
                    resource: light_indices_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 9,
                    resource: cluster_uniforms_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 10,
                    resource: area_lights_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 11,
                    resource: area_uniforms_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 12,
                    resource: prev_joint_matrices_buffer.as_entire_binding(),
                },
            ],
        });

        let cull_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Skinned Mesh Cull Bind Group (Per-World)"),
            layout: cull_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: cull_objects_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: indirect_commands_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: visible_indices_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: cull_uniforms_buffer.as_entire_binding(),
                },
            ],
        });

        let cluster_bounds_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Skinned Mesh Cluster Bounds Bind Group (Per-World)"),
            layout: cluster_bounds_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: cluster_uniforms_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: cluster_bounds_buffer.as_entire_binding(),
                },
            ],
        });

        let dense_capacity_size = 128;
        let dense_capacity_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Skinned Mesh Dense Capacity Buffer (Per-World)"),
            size: (std::mem::size_of::<u32>() * dense_capacity_size) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let command_map_size = 128;
        let command_map_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Skinned Mesh Command Map Buffer (Per-World)"),
            size: (std::mem::size_of::<u32>() * command_map_size) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let batch_meta_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Skinned Mesh Batch Meta Buffer (Per-World)"),
            size: (std::mem::size_of::<u32>() * 2) as u64,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        Self {
            materials_buffer,
            materials_buffer_size,
            lights_buffer,
            lights_buffer_size,
            object_buffer,
            object_buffer_size,
            custom_data_buffer,
            custom_data_buffer_size,
            cluster_uniforms_buffer,
            light_grid_buffer,
            light_grid_reset_buffer,
            light_indices_buffer,
            cull_objects_buffer,
            cull_objects_buffer_size,
            indirect_commands_buffer,
            indirect_commands_buffer_size,
            visible_indices_buffer,
            visible_indices_buffer_size,
            cull_uniforms_buffer,
            instance_bind_group,
            cull_bind_group,
            cluster_bounds_bind_group,
            cluster_assign_bind_group: None,
            area_lights_buffer,
            area_uniforms_buffer,
            dense_capacity_buffer,
            dense_capacity_size,
            command_map_buffer,
            command_map_size,
            batch_meta_buffer,
            batch_bind_group: None,
        }
    }
}

/// A run of same-mesh skinned instances that share one instanced draw
/// call. `first_instance` indexes the object/custom-data buffers, which
/// are ordered so that each group's instances are contiguous.
#[derive(Clone, Copy)]
pub(super) struct SkinnedDrawGroup {
    pub mesh_id: u32,
    pub first_instance: u32,
    pub instance_count: u32,
}

pub(super) struct SkinnedWorldRenderState {
    pub skinned_entities: Vec<freecs::Entity>,
    pub opaque_draw_groups: Vec<SkinnedDrawGroup>,
    pub transparent_draw_groups: Vec<SkinnedDrawGroup>,
    pub cached_cull_objects: Vec<super::types::SkinnedCullObject>,
    pub cached_bounds_inputs: Vec<(usize, usize, f32)>,
    pub cached_materials_generation: u64,
    pub cached_skinned_objects_generation: u64,

    pub last_used_frame: u64,

    pub gpu_buffers: Option<SkinnedWorldGpuBuffers>,

    pub ibl_brdf_lut_view: Option<wgpu::TextureView>,
    pub ibl_irradiance_view: Option<wgpu::TextureView>,
    pub ibl_prefiltered_view: Option<wgpu::TextureView>,
    pub ibl_irradiance_b_view: Option<wgpu::TextureView>,
    pub ibl_prefiltered_b_view: Option<wgpu::TextureView>,
    pub ibl_blend_factor: f32,
}

impl SkinnedWorldRenderState {
    pub fn new() -> Self {
        Self {
            skinned_entities: Vec::new(),
            opaque_draw_groups: Vec::new(),
            transparent_draw_groups: Vec::new(),
            cached_cull_objects: Vec::new(),
            cached_bounds_inputs: Vec::new(),
            cached_materials_generation: u64::MAX,
            cached_skinned_objects_generation: u64::MAX,
            last_used_frame: 0,
            gpu_buffers: None,
            ibl_brdf_lut_view: None,
            ibl_irradiance_view: None,
            ibl_prefiltered_view: None,
            ibl_irradiance_b_view: None,
            ibl_prefiltered_b_view: None,
            ibl_blend_factor: 0.0,
        }
    }
}

impl Default for SkinnedWorldRenderState {
    fn default() -> Self {
        Self::new()
    }
}
