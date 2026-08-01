use nightshade_wgpu::backend::build;
use nightshade_wgpu::wgpu;
use wgpu::util::DeviceExt;

use super::types::{
    AreaLightData, AreaUniforms, ClusterUniforms, LightData, LightGrid, MAX_AREA_LIGHTS,
    MAX_LIGHTS_PER_CLUSTER, MaterialData, SkinnedBoundsInput, SkinnedCullObject,
    SkinnedCullUniforms, SkinnedDrawIndexedIndirect, SkinnedObjectData, TOTAL_CLUSTERS,
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
    pub bounds_inputs_buffer: wgpu::Buffer,
    pub bounds_inputs_buffer_size: usize,
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
        let cull_objects_buffer = build::storage(
            device,
            "Skinned Mesh Cull Objects Buffer (Per-World)",
            (std::mem::size_of::<SkinnedCullObject>() * cull_objects_buffer_size) as u64,
        );

        let bounds_inputs_buffer_size = INITIAL_OBJECTS;
        let bounds_inputs_buffer = build::storage(
            device,
            "Skinned Mesh Bounds Inputs Buffer (Per-World)",
            (std::mem::size_of::<SkinnedBoundsInput>() * bounds_inputs_buffer_size) as u64,
        );

        let indirect_commands_buffer_size = INITIAL_COMMANDS;
        let indirect_commands_buffer = build::buffer(
            device,
            "Skinned Mesh Indirect Commands Buffer (Per-World)",
            (std::mem::size_of::<SkinnedDrawIndexedIndirect>() * indirect_commands_buffer_size)
                as u64,
            wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::INDIRECT
                | wgpu::BufferUsages::COPY_DST,
        );

        let visible_indices_buffer_size = INITIAL_OBJECTS;
        let visible_indices_buffer = build::storage(
            device,
            "Skinned Mesh Visible Indices Buffer (Per-World)",
            (std::mem::size_of::<u32>() * visible_indices_buffer_size) as u64,
        );

        let cull_uniforms_buffer = build::uniform(
            device,
            "Skinned Mesh Cull Uniforms Buffer (Per-World)",
            std::mem::size_of::<SkinnedCullUniforms>() as u64,
        );
        let materials_buffer_size = INITIAL_MATERIALS;
        let materials_buffer = build::storage(
            device,
            "Skinned Mesh Materials Buffer (Per-World)",
            (std::mem::size_of::<MaterialData>() * materials_buffer_size) as u64,
        );

        let lights_buffer_size = INITIAL_LIGHTS;
        let lights_buffer = build::storage(
            device,
            "Skinned Mesh Lights Buffer (Per-World)",
            (std::mem::size_of::<LightData>() * lights_buffer_size) as u64,
        );

        let object_buffer_size = INITIAL_OBJECTS;
        let object_buffer = build::storage(
            device,
            "Skinned Mesh Object Buffer (Per-World)",
            (std::mem::size_of::<SkinnedObjectData>() * object_buffer_size) as u64,
        );

        let custom_data_buffer_size = INITIAL_OBJECTS;
        let custom_data_buffer = build::storage(
            device,
            "Skinned Mesh Custom Data Buffer (Per-World)",
            (std::mem::size_of::<[f32; 4]>() * custom_data_buffer_size) as u64,
        );

        let cluster_uniforms_buffer = build::uniform(
            device,
            "Skinned Cluster Uniforms Buffer (Per-World)",
            std::mem::size_of::<ClusterUniforms>() as u64,
        );

        let light_grid_buffer = build::storage(
            device,
            "Skinned Light Grid Buffer (Per-World)",
            (std::mem::size_of::<LightGrid>() * TOTAL_CLUSTERS as usize) as u64,
        );

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

        let light_indices_buffer = build::storage(
            device,
            "Skinned Light Indices Buffer (Per-World)",
            (std::mem::size_of::<u32>() * TOTAL_CLUSTERS as usize * MAX_LIGHTS_PER_CLUSTER as usize)
                as u64,
        );

        let area_lights_buffer = build::storage(
            device,
            "Skinned Area Lights Buffer (Per-World)",
            (std::mem::size_of::<AreaLightData>() * MAX_AREA_LIGHTS) as u64,
        );

        let area_uniforms_buffer = build::uniform(
            device,
            "Skinned Area Uniforms Buffer (Per-World)",
            std::mem::size_of::<AreaUniforms>() as u64,
        );

        let instance_bind_group = build::binding(
            device,
            "Skinned Mesh Instance Bind Group (Per-World)",
            instance_bind_group_layout,
            vec![
                materials_buffer.as_entire_binding(),
                object_buffer.as_entire_binding(),
                lights_buffer.as_entire_binding(),
                joint_matrices_buffer.as_entire_binding(),
                custom_data_buffer.as_entire_binding(),
                morph_displacement_buffer.as_entire_binding(),
                visible_indices_buffer.as_entire_binding(),
                light_grid_buffer.as_entire_binding(),
                light_indices_buffer.as_entire_binding(),
                cluster_uniforms_buffer.as_entire_binding(),
                area_lights_buffer.as_entire_binding(),
                area_uniforms_buffer.as_entire_binding(),
                prev_joint_matrices_buffer.as_entire_binding(),
            ],
        );

        let cull_bind_group = build::binding(
            device,
            "Skinned Mesh Cull Bind Group (Per-World)",
            cull_bind_group_layout,
            vec![
                cull_objects_buffer.as_entire_binding(),
                indirect_commands_buffer.as_entire_binding(),
                visible_indices_buffer.as_entire_binding(),
                cull_uniforms_buffer.as_entire_binding(),
            ],
        );

        let cluster_bounds_bind_group = build::binding(
            device,
            "Skinned Mesh Cluster Bounds Bind Group (Per-World)",
            cluster_bounds_bind_group_layout,
            vec![
                cluster_uniforms_buffer.as_entire_binding(),
                cluster_bounds_buffer.as_entire_binding(),
            ],
        );

        let dense_capacity_size = 128;
        let dense_capacity_buffer = build::storage(
            device,
            "Skinned Mesh Dense Capacity Buffer (Per-World)",
            (std::mem::size_of::<u32>() * dense_capacity_size) as u64,
        );
        let command_map_size = 128;
        let command_map_buffer = build::storage(
            device,
            "Skinned Mesh Command Map Buffer (Per-World)",
            (std::mem::size_of::<u32>() * command_map_size) as u64,
        );
        let batch_meta_buffer = build::buffer(
            device,
            "Skinned Mesh Batch Meta Buffer (Per-World)",
            (std::mem::size_of::<u32>() * 2) as u64,
            wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
        );

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
            bounds_inputs_buffer,
            bounds_inputs_buffer_size,
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

#[derive(Clone, Copy)]
pub(super) struct SkinnedDrawGroup {
    pub mesh_id: u32,
    pub first_instance: u32,
    pub instance_count: u32,
}

pub(super) struct SkinnedWorldRenderState {
    pub skinned_entities: Vec<nightshade::ecs::Entity>,
    pub opaque_draw_groups: Vec<SkinnedDrawGroup>,
    pub transparent_draw_groups: Vec<SkinnedDrawGroup>,
    pub cached_cull_objects: Vec<super::types::SkinnedCullObject>,
    pub cached_bounds_inputs: Vec<(usize, usize, f32)>,
    pub cached_indirect_commands: Vec<super::types::SkinnedDrawIndexedIndirect>,
    pub cached_materials_generation: u64,
    pub cached_skinned_generation: u64,

    pub last_used_frame: u64,

    pub gpu_buffers: Option<SkinnedWorldGpuBuffers>,
}

impl SkinnedWorldRenderState {
    pub fn new() -> Self {
        Self {
            skinned_entities: Vec::new(),
            opaque_draw_groups: Vec::new(),
            transparent_draw_groups: Vec::new(),
            cached_cull_objects: Vec::new(),
            cached_bounds_inputs: Vec::new(),
            cached_indirect_commands: Vec::new(),
            cached_materials_generation: u64::MAX,
            cached_skinned_generation: u64::MAX,
            last_used_frame: 0,
            gpu_buffers: None,
        }
    }
}

impl Default for SkinnedWorldRenderState {
    fn default() -> Self {
        Self::new()
    }
}
