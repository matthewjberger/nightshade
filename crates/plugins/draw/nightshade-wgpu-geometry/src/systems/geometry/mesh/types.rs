use std::collections::HashMap;

pub(super) use nightshade_wgpu_shading::backend::projection::{
    AreaLightData, AreaUniforms, LightData, MAX_AREA_LIGHTS,
};

pub(super) use nightshade_wgpu_shading::backend::scene_uniforms::SceneUniforms;

use nightshade::ecs::Entity;

pub(super) const INITIAL_INSTANCES: usize = 1_000;
pub const MAX_INSTANCES: usize = 1_000_000;
pub(super) const INITIAL_VERTEX_BUFFER_SIZE: usize = 10_000;
pub(super) const INITIAL_INDEX_BUFFER_SIZE: usize = 30_000;
pub(super) const INITIAL_MATERIALS: usize = 100;
pub(super) const INITIAL_LIGHTS: usize = 32;
pub(super) use nightshade_wgpu_shading::backend::projection::MAX_LIGHTS;
pub(super) const BUFFER_GROWTH_FACTOR: f32 = 2.0;

pub(super) fn grown_buffer_size(needed: usize) -> usize {
    let grown = (needed as f32 * BUFFER_GROWTH_FACTOR).ceil() as usize;
    std::cmp::max(needed, std::cmp::min(grown, MAX_INSTANCES))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_small_request_grows_by_the_factor() {
        assert_eq!(grown_buffer_size(100), 200);
    }

    #[test]
    fn the_cap_bounds_the_speculation() {
        assert_eq!(grown_buffer_size(MAX_INSTANCES - 1), MAX_INSTANCES);
    }

    #[test]
    fn a_request_over_the_cap_is_still_honored() {
        assert_eq!(grown_buffer_size(MAX_INSTANCES + 4), MAX_INSTANCES + 4);
    }

    #[test]
    fn nothing_is_ever_sized_below_what_it_must_hold() {
        for needed in [0, 1, 7, 1_000, MAX_INSTANCES, MAX_INSTANCES * 3] {
            assert!(grown_buffer_size(needed) >= needed);
        }
    }
}
pub(super) const BUFFER_SHRINK_THRESHOLD: f32 = 0.1;
pub(super) const COMPACTION_UTILIZATION_DROP_THRESHOLD: f32 = 0.2;
pub(super) const COMPACTION_MIN_RECLAIM_BYTES: u64 = 16 * 1024 * 1024;
pub(super) const COMPACTION_COOLDOWN_FRAMES: u64 = 600;

pub(super) const NUM_DRAW_CLASSES: usize = 10;

pub(super) const MAX_BATCHES_PER_CLASS: usize = 1024;

pub use nightshade_wgpu_shading::backend::cluster::{
    CLUSTER_GRID_X, CLUSTER_GRID_Y, CLUSTER_GRID_Z, ClusterBounds, ClusterUniforms, LightGrid,
    MAX_LIGHTS_PER_CLUSTER, TOTAL_CLUSTERS, cluster_slice_near,
};

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
    pub padding: [u32; 2],
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
    pub(super) debug_view_exempt: u32,
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
    pub(super) pad_culling: [u32; 3],
}

pub use nightshade_wgpu_shading::backend::material_gpu::MaterialData;

#[derive(Debug, Clone, Copy)]
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
    pub(super) padding: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub(super) struct MorphDisplacement {
    pub(super) position: [f32; 3],
    pub(super) padding_0: f32,
    pub(super) normal: [f32; 3],
    pub(super) padding_1: f32,
    pub(super) tangent: [f32; 3],
    pub(super) padding_2: f32,
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
    pub entity_to_slot: HashMap<Entity, u32>,
    pub slot_to_entity: Vec<Option<Entity>>,
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
