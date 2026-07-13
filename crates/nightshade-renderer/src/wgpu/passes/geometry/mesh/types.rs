use std::collections::HashMap;

pub(super) use super::super::projection::{
    AreaLightData, AreaUniforms, LightData, MAX_AREA_LIGHTS,
};

use crate::entity::RenderEntity;

pub(super) const INITIAL_INSTANCES: usize = 1_000;
pub const MAX_INSTANCES: usize = 1_000_000;
pub(super) const INITIAL_VERTEX_BUFFER_SIZE: usize = 10_000;
pub(super) const INITIAL_INDEX_BUFFER_SIZE: usize = 30_000;
pub(super) const INITIAL_MATERIALS: usize = 100;
pub(super) const INITIAL_LIGHTS: usize = 32;
pub(super) const MAX_LIGHTS: usize = 1024;
pub(super) const BUFFER_GROWTH_FACTOR: f32 = 2.0;
pub(super) const BUFFER_SHRINK_THRESHOLD: f32 = 0.1;
pub(super) const COMPACTION_UTILIZATION_DROP_THRESHOLD: f32 = 0.2;
pub(super) const COMPACTION_MIN_RECLAIM_BYTES: u64 = 16 * 1024 * 1024;

/// Number of pipeline-class draw ranges the GPU indirect-compaction pass
/// processes: world opaque, opaque double-sided, transparent, the three
/// overlay variants, then the three instanced variants.
pub(super) const NUM_DRAW_CLASSES: usize = 9;

/// Floor for the per-class batch-slot capacity in the GPU-driven batch build.
/// The GPU writes each class's batches into `[class * cap, ...)`; `cap` grows
/// from this floor to a CPU-derived upper bound on the per-class batch count so
/// the draw submission needs no CPU knowledge of the actual per-class counts.
pub(super) const MAX_BATCHES_PER_CLASS: usize = 1024;

pub use super::super::cluster::{
    CLUSTER_GRID_X, CLUSTER_GRID_Y, CLUSTER_GRID_Z, ClusterBounds, ClusterUniforms, LightGrid,
    MAX_LIGHTS_PER_CLUSTER, TOTAL_CLUSTERS, cluster_slice_near,
};

#[repr(C, align(256))]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub(super) struct MeshUniforms {
    pub(super) view: [[f32; 4]; 4],
    pub(super) projection: [[f32; 4]; 4],
    pub(super) camera_position: [f32; 4],
    pub(super) num_lights: [u32; 4],
    pub(super) ambient_light: [f32; 4],
    pub(super) light_view_projection: [[f32; 4]; 4],
    pub(super) shadow_bias: f32,
    pub(super) shadows_enabled: f32,
    pub(super) global_unlit: f32,
    pub(super) shadow_normal_bias: f32,
    pub(super) snap_resolution: [f32; 2],
    pub(super) snap_enabled: u32,
    pub(super) affine_enabled: u32,
    pub(super) fog_color: [f32; 3],
    pub(super) fog_enabled: u32,
    pub(super) fog_start: f32,
    pub(super) fog_end: f32,
    pub(super) cascade_count: u32,
    pub(super) directional_light_size: f32,
    pub(super) cascade_view_projections: [[[f32; 4]; 4]; crate::wgpu::passes::NUM_SHADOW_CASCADES],
    pub(super) cascade_split_distances: [f32; crate::wgpu::passes::NUM_SHADOW_CASCADES],
    pub(super) cascade_atlas_offsets: [[f32; 4]; crate::wgpu::passes::NUM_SHADOW_CASCADES],
    pub(super) cascade_atlas_scale: [f32; 4],
    pub(super) time: f32,
    pub(super) pbr_debug_mode: u32,
    pub(super) texture_debug_stripes: u32,
    pub(super) texture_debug_stripes_speed: f32,
    pub(super) directional_light_direction: [f32; 4],
    pub(super) ibl_blend_factor: f32,
    pub(super) oit_z_scale: f32,
    pub(super) _pad_pre_flat: [f32; 2],
    pub(super) flat_color: [f32; 4],
    pub(super) prev_view_projection: [[f32; 4]; 4],
    pub(super) cur_view_projection: [[f32; 4]; 4],
    pub(super) _padding3: [f32; 44],
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub(super) struct ModelMatrix {
    pub(super) model: [[f32; 4]; 4],
    pub(super) normal_matrix: [[f32; 4]; 3],
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub(super) struct InstancedTransformComputeUniforms {
    pub parent_transform: [[f32; 4]; 4],
    pub instance_count: u32,
    pub output_offset: u32,
    pub _pad: [u32; 2],
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub(super) struct ObjectData {
    pub(super) transform_index: u32,
    pub(super) mesh_id: u32,
    pub(super) material_id: u32,
    pub(super) batch_id: u32,
    pub(super) morph_weights: [f32; 8],
    pub(super) morph_target_count: u32,
    pub(super) morph_displacement_offset: u32,
    pub(super) mesh_vertex_offset: u32,
    pub(super) mesh_vertex_count: u32,
    pub(super) entity_id: u32,
    pub(super) is_overlay: u32,
    pub(super) skip_occlusion: u32,
    pub(super) flip_winding: u32,
    pub(super) culling_mask: u32,
    pub(super) visible: u32,
    pub(super) pipeline_class: u32,
    pub(super) _pad_culling: [u32; 1],
}

#[repr(C, align(16))]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub(super) struct CullingUniforms {
    pub(super) frustum_planes: [[f32; 4]; 6],
    pub(super) view_projection: [[f32; 4]; 4],
    pub(super) occluder_view_projection: [[f32; 4]; 4],
    pub(super) screen_size: [f32; 2],
    pub(super) object_count: u32,
    pub(super) min_screen_pixel_size: f32,
    pub(super) projection_scale_y: f32,
    pub(super) camera_culling_mask: u32,
    pub(super) hiz_mip_count: f32,
    pub(super) occlusion_enabled: u32,
    pub(super) frustum_enabled: u32,
    pub(super) _pad_culling: [u32; 3],
}

pub use super::super::material_gpu::MaterialData;

#[derive(Debug, Clone)]
pub(super) struct MeshData {
    pub(super) vertex_offset: u32,
    pub(super) vertex_count: u32,
    pub(super) index_offset: u32,
    pub(super) index_count: u32,
    pub(super) morph_displacement_offset: u32,
    pub(super) morph_target_count: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub(super) struct MeshBoundsData {
    pub(super) center: [f32; 3],
    pub(super) radius: f32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub(super) struct BatchDesc {
    pub(super) mesh_geo_id: u32,
    pub(super) capacity: u32,
}

/// One distinct `(pipeline_class, mesh_id, material_id)` combo and the base
/// batch slot it maps to. Kept sorted lexicographically by the first three
/// fields so the GPU assign pass can binary search it.
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, bytemuck::Pod, bytemuck::Zeroable)]
pub(super) struct BatchKey {
    pub(super) pipeline_class: u32,
    pub(super) mesh_id: u32,
    pub(super) material_id: u32,
    pub(super) base_slot: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub(super) struct MeshGeo {
    pub(super) index_count: u32,
    pub(super) first_index: u32,
    pub(super) base_vertex: i32,
    pub(super) _pad: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub(super) struct MeshLodGeoData {
    pub(super) lod_count: u32,
    pub(super) geo: [u32; 4],
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub(super) struct MorphDisplacement {
    pub(super) position: [f32; 3],
    pub(super) _pad0: f32,
    pub(super) normal: [f32; 3],
    pub(super) _pad1: f32,
    pub(super) tangent: [f32; 3],
    pub(super) _pad2: f32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub(super) struct MeshLodInfo {
    pub(super) lod_count: u32,
    pub(super) thresholds: [f32; 3],
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub(super) struct DrawIndexedIndirect {
    pub(super) index_count: u32,
    pub(super) instance_count: u32,
    pub(super) first_index: u32,
    pub(super) base_vertex: i32,
    pub(super) first_instance: u32,
}

#[derive(Debug)]
pub struct GpuEntityRegistry {
    pub entity_to_slot: HashMap<RenderEntity, u32>,
    pub slot_to_entity: Vec<Option<RenderEntity>>,
    pub free_slots: Vec<u32>,
    pub slot_count: u32,
}

impl Default for GpuEntityRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl GpuEntityRegistry {
    pub fn new() -> Self {
        Self {
            entity_to_slot: HashMap::new(),
            slot_to_entity: Vec::new(),
            free_slots: Vec::new(),
            slot_count: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.entity_to_slot.len()
    }

    pub fn clear(&mut self) {
        self.entity_to_slot.clear();
        self.slot_to_entity.clear();
        self.free_slots.clear();
        self.slot_count = 0;
    }
}
