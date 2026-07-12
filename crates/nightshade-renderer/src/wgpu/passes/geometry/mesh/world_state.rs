use freecs::Entity;
use std::collections::HashMap;
use wgpu::util::DeviceExt;

use super::types::{
    AreaLightData, AreaUniforms, ClusterUniforms, DrawIndexedIndirect, GpuEntityRegistry,
    INITIAL_INSTANCES, INITIAL_LIGHTS, INITIAL_MATERIALS, LightData, LightGrid, MAX_AREA_LIGHTS,
    MAX_LIGHTS_PER_CLUSTER, MaterialData, ModelMatrix, NUM_DRAW_CLASSES, ObjectData,
    TOTAL_CLUSTERS,
};

pub(super) type BatchRange = (u32, u32, u32, u32);

pub(super) const FULL_UPLOAD_RATIO_THRESHOLD: f32 = 0.3;

#[derive(Debug, Default)]
pub(super) struct WorldUploadRanges {
    pub transform_ranges: Vec<(u32, u32)>,
    pub object_ranges: Vec<(u32, u32)>,
    pub custom_data_ranges: Vec<(u32, u32)>,
    pub full_transforms: bool,
    pub full_objects: bool,
    pub full_custom_data: bool,
}

pub(super) fn diff_into_ranges<T: PartialEq>(prev: &[T], curr: &[T]) -> (Vec<(u32, u32)>, bool) {
    if prev.len() != curr.len() {
        return (Vec::new(), true);
    }
    let mut ranges: Vec<(u32, u32)> = Vec::new();
    let mut start: Option<u32> = None;
    for (index, _) in curr.iter().enumerate() {
        if prev[index] != curr[index] {
            if start.is_none() {
                start = Some(index as u32);
            }
        } else if let Some(open) = start.take() {
            ranges.push((open, index as u32));
        }
    }
    if let Some(open) = start {
        ranges.push((open, curr.len() as u32));
    }
    let dirty_count: usize = ranges.iter().map(|(s, e)| (e - s) as usize).sum();
    let denominator = curr.len().max(1);
    if (dirty_count as f32 / denominator as f32) > FULL_UPLOAD_RATIO_THRESHOLD {
        return (Vec::new(), true);
    }
    (ranges, false)
}

#[cfg(debug_assertions)]
pub(super) fn validate_upload_ranges<T: PartialEq + Clone>(
    prev: &[T],
    curr: &[T],
    ranges: &[(u32, u32)],
    full_upload: bool,
    label: &str,
) {
    if full_upload {
        return;
    }
    if prev.len() != curr.len() {
        panic!(
            "snapshot diff validation failed for {label}: prev.len()={} curr.len()={} but full_upload=false",
            prev.len(),
            curr.len(),
        );
    }
    let mut reconstructed = prev.to_vec();
    for (start, end) in ranges {
        let s = *start as usize;
        let e = *end as usize;
        reconstructed[s..e].clone_from_slice(&curr[s..e]);
    }
    for (index, (got, expected)) in reconstructed.iter().zip(curr.iter()).enumerate() {
        if got != expected {
            panic!(
                "snapshot diff validation failed for {label} at index {index}: ranges did not cover a real difference",
            );
        }
    }
}

#[cfg(not(debug_assertions))]
pub(super) fn validate_upload_ranges<T: PartialEq + Clone>(
    _prev: &[T],
    _curr: &[T],
    _ranges: &[(u32, u32)],
    _full_upload: bool,
    _label: &str,
) {
}

pub(super) struct WorldGpuBuffers {
    pub materials_buffer: wgpu::Buffer,
    pub materials_buffer_size: usize,
    pub transform_buffer: wgpu::Buffer,
    pub transform_buffer_size: usize,
    pub prev_transform_buffer: wgpu::Buffer,
    pub object_buffer: wgpu::Buffer,
    pub object_buffer_size: usize,
    pub lights_buffer: wgpu::Buffer,
    pub lights_buffer_size: usize,
    pub custom_data_buffer: wgpu::Buffer,
    pub custom_data_buffer_size: usize,
    pub indirect_buffer: wgpu::Buffer,
    pub indirect_buffer_size: usize,
    pub indirect_reset_buffer: wgpu::Buffer,
    pub visible_indices_buffer: wgpu::Buffer,
    pub instance_bind_group: wgpu::BindGroup,
    pub culling_bind_group: Option<wgpu::BindGroup>,
    pub culling_bind_group_frustum: Option<wgpu::BindGroup>,
    pub draw_count_buffer: wgpu::Buffer,
    pub class_ranges_buffer: wgpu::Buffer,
    pub compaction_bind_group: Option<wgpu::BindGroup>,
    pub batch_descs_buffer: wgpu::Buffer,
    pub indirect_build_bind_group: Option<wgpu::BindGroup>,
    pub batch_assign_bind_group: Option<wgpu::BindGroup>,
    pub batch_key_buffer: wgpu::Buffer,
    pub batch_key_buffer_size: usize,
    pub batch_key_count: u32,
    pub cluster_uniforms_buffer: wgpu::Buffer,
    pub light_grid_buffer: wgpu::Buffer,
    pub light_grid_reset_buffer: wgpu::Buffer,
    pub light_indices_buffer: wgpu::Buffer,
    pub cluster_bounds_bind_group: wgpu::BindGroup,
    pub cluster_assign_bind_group: Option<wgpu::BindGroup>,
    pub area_lights_buffer: wgpu::Buffer,
    pub area_uniforms_buffer: wgpu::Buffer,
    pub instanced_local_matrix_buffer: wgpu::Buffer,
    pub instanced_local_matrix_buffer_size: usize,
    pub instanced_compute_bind_group: Option<wgpu::BindGroup>,
    pub material_flags_buffer: wgpu::Buffer,
    pub material_flags_buffer_size: usize,
    pub dense_capacity_buffer: wgpu::Buffer,
    pub dense_capacity_size: usize,
    pub batch_meta_buffer: wgpu::Buffer,
    pub gpu_batch_bind_group: Option<wgpu::BindGroup>,
}

impl WorldGpuBuffers {
    pub fn new(
        device: &wgpu::Device,
        instance_bind_group_layout: &wgpu::BindGroupLayout,
        cluster_bounds_bind_group_layout: &wgpu::BindGroupLayout,
        cluster_bounds_buffer: &wgpu::Buffer,
        morph_displacement_buffer: &wgpu::Buffer,
    ) -> Self {
        let materials_buffer_size = INITIAL_MATERIALS;
        let materials_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Mesh Materials Buffer (Per-World)"),
            size: (std::mem::size_of::<MaterialData>() * materials_buffer_size) as u64,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let transform_buffer_size = INITIAL_INSTANCES;
        let transform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Mesh Transform Buffer (Per-World)"),
            size: (std::mem::size_of::<ModelMatrix>() * transform_buffer_size) as u64,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let prev_transform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Mesh Previous Transform Buffer (Per-World)"),
            size: (std::mem::size_of::<ModelMatrix>() * transform_buffer_size) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let object_buffer_size = INITIAL_INSTANCES;
        let object_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Mesh Object Buffer (Per-World)"),
            size: (std::mem::size_of::<ObjectData>() * object_buffer_size) as u64,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let lights_buffer_size = INITIAL_LIGHTS;
        let lights_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Mesh Lights Buffer (Per-World)"),
            size: (std::mem::size_of::<LightData>() * lights_buffer_size) as u64,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let custom_data_buffer_size = INITIAL_INSTANCES;
        let custom_data_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Mesh Custom Data Buffer (Per-World)"),
            size: (std::mem::size_of::<[f32; 4]>() * custom_data_buffer_size) as u64,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let indirect_buffer_size = INITIAL_INSTANCES;
        let indirect_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Indirect Draw Buffer (Per-World)"),
            size: (std::mem::size_of::<DrawIndexedIndirect>() * indirect_buffer_size) as u64,
            usage: wgpu::BufferUsages::INDIRECT
                | wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let indirect_reset_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Indirect Reset Buffer (Per-World)"),
            size: (std::mem::size_of::<DrawIndexedIndirect>() * indirect_buffer_size) as u64,
            usage: wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });

        let batch_descs_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Batch Descriptor Buffer (Per-World)"),
            size: (std::mem::size_of::<super::types::BatchDesc>() * indirect_buffer_size) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let batch_key_buffer_size = indirect_buffer_size;
        let batch_key_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Batch Key Buffer (Per-World)"),
            size: (std::mem::size_of::<super::types::BatchKey>() * batch_key_buffer_size) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let visible_indices_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Visible Indices Buffer (Per-World)"),
            size: (std::mem::size_of::<u32>() * INITIAL_INSTANCES) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let draw_count_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Draw Count Buffer (Per-World)"),
            size: (std::mem::size_of::<u32>() * NUM_DRAW_CLASSES) as u64,
            usage: wgpu::BufferUsages::INDIRECT
                | wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let class_ranges_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Draw Class Ranges Buffer (Per-World)"),
            size: (std::mem::size_of::<[u32; 2]>() * NUM_DRAW_CLASSES) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let cluster_uniforms_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Cluster Uniforms Buffer (Per-World)"),
            size: std::mem::size_of::<ClusterUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let light_grid_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Light Grid Buffer (Per-World)"),
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
                label: Some("Light Grid Reset Buffer (Per-World)"),
                contents: bytemuck::cast_slice(&light_grid_reset_data),
                usage: wgpu::BufferUsages::COPY_SRC,
            });

        let light_indices_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Light Indices Buffer (Per-World)"),
            size: (std::mem::size_of::<u32>()
                * TOTAL_CLUSTERS as usize
                * MAX_LIGHTS_PER_CLUSTER as usize) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let area_lights_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Area Lights Buffer (Per-World)"),
            size: (std::mem::size_of::<AreaLightData>() * MAX_AREA_LIGHTS) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let area_uniforms_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Area Uniforms Buffer (Per-World)"),
            size: std::mem::size_of::<AreaUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let instanced_local_matrix_buffer_size = INITIAL_INSTANCES;
        let instanced_local_matrix_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Instanced Local Matrix Buffer (Per-World)"),
            size: (std::mem::size_of::<[[f32; 4]; 4]>() * instanced_local_matrix_buffer_size)
                as u64,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let material_flags_buffer_size = INITIAL_MATERIALS;
        let material_flags_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Mesh Material Flags Buffer (Per-World)"),
            size: (std::mem::size_of::<u32>() * material_flags_buffer_size) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let dense_capacity_size = 18 * INITIAL_MATERIALS;
        let dense_capacity_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Mesh Dense Capacity Buffer (Per-World)"),
            size: (std::mem::size_of::<u32>() * dense_capacity_size) as u64,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let batch_meta_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Mesh Batch Meta Buffer (Per-World)"),
            size: (std::mem::size_of::<u32>() * 3 * NUM_DRAW_CLASSES) as u64,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::INDIRECT,
            mapped_at_creation: false,
        });

        let cluster_bounds_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Cluster Bounds Bind Group (Per-World)"),
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

        let instance_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Mesh Instance Bind Group (Per-World)"),
            layout: instance_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: transform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: materials_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: object_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: lights_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: visible_indices_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: custom_data_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: morph_displacement_buffer.as_entire_binding(),
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
                    resource: prev_transform_buffer.as_entire_binding(),
                },
            ],
        });

        Self {
            materials_buffer,
            materials_buffer_size,
            transform_buffer,
            transform_buffer_size,
            prev_transform_buffer,
            object_buffer,
            object_buffer_size,
            lights_buffer,
            lights_buffer_size,
            custom_data_buffer,
            custom_data_buffer_size,
            indirect_buffer,
            indirect_buffer_size,
            indirect_reset_buffer,
            visible_indices_buffer,
            instance_bind_group,
            culling_bind_group: None,
            culling_bind_group_frustum: None,
            draw_count_buffer,
            class_ranges_buffer,
            compaction_bind_group: None,
            batch_descs_buffer,
            indirect_build_bind_group: None,
            batch_assign_bind_group: None,
            batch_key_buffer,
            batch_key_buffer_size,
            batch_key_count: 0,
            cluster_uniforms_buffer,
            light_grid_buffer,
            light_grid_reset_buffer,
            light_indices_buffer,
            cluster_bounds_bind_group,
            cluster_assign_bind_group: None,
            area_lights_buffer,
            area_uniforms_buffer,
            instanced_local_matrix_buffer,
            instanced_local_matrix_buffer_size,
            instanced_compute_bind_group: None,
            material_flags_buffer,
            material_flags_buffer_size,
            dense_capacity_buffer,
            dense_capacity_size,
            batch_meta_buffer,
            gpu_batch_bind_group: None,
        }
    }
}

pub(super) struct WorldRenderState {
    pub gpu_registry: GpuEntityRegistry,
    pub entity_to_transform_index: HashMap<Entity, u32>,
    pub cached_entities: Vec<Entity>,
    pub cached_transforms: Vec<ModelMatrix>,
    pub cached_objects: Vec<ObjectData>,

    pub opaque_instances: Vec<BatchRange>,
    pub opaque_double_sided_instances: Vec<BatchRange>,
    pub transparent_instances: Vec<BatchRange>,
    pub overlay_opaque_instances: Vec<BatchRange>,
    pub overlay_opaque_double_sided_instances: Vec<BatchRange>,
    pub overlay_transparent_instances: Vec<BatchRange>,
    pub instanced_opaque_batches: Vec<BatchRange>,
    pub instanced_opaque_double_sided_batches: Vec<BatchRange>,
    pub instanced_transparent_batches: Vec<BatchRange>,
    pub regular_object_count: u32,
    pub instanced_transform_ranges: HashMap<Entity, (u32, u32)>,
    pub instanced_compute_dirty: bool,

    pub object_count: u32,

    pub indirect_reset_count: usize,

    pub last_camera_hash: u64,
    pub camera_changed: bool,
    pub num_directional_lights: u32,
    pub num_total_lights: u32,

    pub last_used_frame: u64,

    pub gpu_buffers: Option<WorldGpuBuffers>,

    pub ibl_brdf_lut_view: Option<wgpu::TextureView>,
    pub ibl_irradiance_view: Option<wgpu::TextureView>,
    pub ibl_prefiltered_view: Option<wgpu::TextureView>,
    pub ibl_irradiance_b_view: Option<wgpu::TextureView>,
    pub ibl_prefiltered_b_view: Option<wgpu::TextureView>,
    pub ibl_blend_factor: f32,
    pub ibl_dirty: bool,

    pub cached_name_to_material_id: HashMap<String, u32>,
    pub cached_material_map: HashMap<Entity, u32>,
    pub cached_materials_data: Vec<MaterialData>,
    pub cached_custom_data: Vec<[f32; 4]>,
    pub combos: HashMap<(u32, u32, u32), u32>,
    /// Per draw class, the count of leading non-mask batches. The depth prepass
    /// draws only this prefix of each opaque class as a single multi-draw, since
    /// alpha-mask batches are ordered into a contiguous suffix and are excluded
    /// from the depth-only prepass.
    pub prepass_batch_counts: [u32; NUM_DRAW_CLASSES],
    pub cached_transparent_material_ids: std::collections::HashSet<u32>,
    pub cached_double_sided_material_ids: std::collections::HashSet<u32>,
    pub cached_mask_material_ids: std::collections::HashSet<u32>,
    pub frames_since_full_rebuild: u64,
    pub free_slots_by_group: HashMap<(u32, u32), Vec<u32>>,
}

impl WorldRenderState {
    pub fn new() -> Self {
        Self {
            gpu_registry: GpuEntityRegistry::new(),
            entity_to_transform_index: HashMap::new(),
            cached_entities: Vec::new(),
            cached_transforms: Vec::new(),
            cached_objects: Vec::new(),

            opaque_instances: Vec::new(),
            opaque_double_sided_instances: Vec::new(),
            transparent_instances: Vec::new(),
            overlay_opaque_instances: Vec::new(),
            overlay_opaque_double_sided_instances: Vec::new(),
            overlay_transparent_instances: Vec::new(),
            instanced_opaque_batches: Vec::new(),
            instanced_opaque_double_sided_batches: Vec::new(),
            instanced_transparent_batches: Vec::new(),
            regular_object_count: 0,
            instanced_transform_ranges: HashMap::new(),
            instanced_compute_dirty: false,

            object_count: 0,

            indirect_reset_count: 0,

            last_camera_hash: 0,
            camera_changed: false,
            num_directional_lights: 0,
            num_total_lights: 0,

            last_used_frame: 0,

            gpu_buffers: None,

            ibl_brdf_lut_view: None,
            ibl_irradiance_view: None,
            ibl_prefiltered_view: None,
            ibl_irradiance_b_view: None,
            ibl_prefiltered_b_view: None,
            ibl_blend_factor: 0.0,
            ibl_dirty: false,

            cached_name_to_material_id: HashMap::new(),
            cached_material_map: HashMap::new(),
            cached_materials_data: Vec::new(),
            cached_custom_data: Vec::new(),
            combos: HashMap::new(),
            prepass_batch_counts: [0; NUM_DRAW_CLASSES],
            cached_transparent_material_ids: std::collections::HashSet::new(),
            cached_double_sided_material_ids: std::collections::HashSet::new(),
            cached_mask_material_ids: std::collections::HashSet::new(),
            frames_since_full_rebuild: 0,
            free_slots_by_group: HashMap::new(),
        }
    }
}

impl Default for WorldRenderState {
    fn default() -> Self {
        Self::new()
    }
}
