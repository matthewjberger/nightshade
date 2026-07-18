pub const MAX_JOINTS_PER_SKIN: usize = 256;
pub const INITIAL_MAX_SKINS: usize = 100;
pub const INITIAL_MAX_TOTAL_BONES: usize = 5_000;
pub const MAX_SPOTLIGHT_SHADOWS: usize = 64;
pub const SPOTLIGHT_ATLAS_SLOTS_PER_ROW: u32 = 8;

pub const NUM_SHADOW_CASCADES: usize = 4;
pub const CASCADE_ATLAS_SLOTS_PER_ROW: u32 = 2;
pub const CASCADE_ATLAS_SIZE: u32 = if cfg!(target_arch = "wasm32") {
    4096
} else {
    8192
};
pub const CASCADE_SLOT_RESOLUTION: f32 = (CASCADE_ATLAS_SIZE / CASCADE_ATLAS_SLOTS_PER_ROW) as f32;
/// Far distance of each cascade in world units. The near boundaries (10, 40)
/// match the previous close-range tuning so zoomed-in shadows keep their density,
/// while the far reach is pulled in from 500 to 150 and the freed cascade is
/// redistributed into the mid range. This concentrates texel density where an
/// orbit camera actually frames its subject, so shadows stay crisp when the
/// camera pulls back instead of collapsing into the single 150..500 slab.
pub const CASCADE_SPLIT_DISTANCES: [f32; NUM_SHADOW_CASCADES] = [10.0, 40.0, 90.0, 150.0];

pub const CASCADE_REFERENCE_NEAR: f32 = 0.01;

pub fn scale_cascade_splits(camera_near: f32) -> [f32; NUM_SHADOW_CASCADES] {
    // Shrink the cascades for very-near (tiny-scene) cameras, but never grow them
    // past the base distances. An unbounded scale let a moderate near (~0.1) blow
    // cascade 0 out to ~100 units, making every shadow texel huge and blocky.
    let scale = (camera_near / CASCADE_REFERENCE_NEAR).clamp(1e-6, 1.0);
    let mut splits = CASCADE_SPLIT_DISTANCES;
    for split in &mut splits {
        *split *= scale;
    }
    splits
}

pub const MAX_POINT_LIGHT_SHADOWS: usize = 16;
pub const POINT_SHADOW_FACE_SIZE: u32 = 256;
pub const POINT_SHADOW_NUM_FACES: usize = 6;

/// One shadow-casting occluder: which transform it uses, which mesh geometry
/// it draws, and which indirect batch it belongs to. Rebuilt each frame from
/// the shadow caster list. The GPU cull writes `transform_index` into the
/// visible-index buffer for each view the occluder survives.
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ShadowOccluder {
    pub transform_index: u32,
    pub mesh_geo_id: u32,
    pub batch_id: u32,
    pub _pad: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ShadowMeshGeo {
    pub index_count: u32,
    pub first_index: u32,
    pub base_vertex: i32,
    pub _pad: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ShadowMeshBounds {
    pub center: [f32; 3],
    pub radius: f32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ShadowDrawIndexedIndirect {
    pub index_count: u32,
    pub instance_count: u32,
    pub first_index: u32,
    pub base_vertex: i32,
    pub first_instance: u32,
}

/// Per-view culling parameters. `indirect_offset` selects the view's run of
/// per-batch indirect commands; the visible-index offsets are baked into each
/// command's `first_instance` so the cull only needs the frustum and counts.
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ShadowCullUniforms {
    pub frustum_planes: [[f32; 4]; 6],
    pub occluder_count: u32,
    pub indirect_offset: u32,
    pub _pad0: u32,
    pub _pad1: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SkinnedShadowObjectData {
    pub transform_index: u32,
    pub mesh_id: u32,
    pub material_id: u32,
    pub joint_offset: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SkinnedShadowCullObject {
    pub bounds: [f32; 4],
    pub command_index: u32,
    pub _pad: [u32; 3],
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ShadowUniforms {
    pub light_view_projection: [[f32; 4]; 4],
    pub _padding: [[f32; 4]; 4],
    pub _padding2: [[f32; 4]; 4],
    pub _padding3: [[f32; 4]; 4],
    pub _padding4: [f32; 4],
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CascadeData {
    pub view_projections: [[[f32; 4]; 4]; NUM_SHADOW_CASCADES],
    pub split_distances: [f32; NUM_SHADOW_CASCADES],
    pub atlas_offsets: [[f32; 2]; NUM_SHADOW_CASCADES],
    pub atlas_scale: [f32; 2],
    pub _padding: [f32; 2],
}

#[derive(Debug, Clone)]
pub struct SpotlightShadowSlot {
    pub entity: freecs::Entity,
    pub view_projection: nalgebra_glm::Mat4,
    pub slot_index: u32,
    pub bias: f32,
    pub atlas_x_pixels: u32,
    pub atlas_y_pixels: u32,
    pub atlas_size_pixels: u32,
    pub clean: bool,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SpotlightShadowData {
    pub view_projection: [[f32; 4]; 4],
    pub atlas_offset: [f32; 2],
    pub atlas_scale: [f32; 2],
    pub bias: f32,
    pub _padding: [f32; 3],
}

#[derive(Debug, Clone)]
pub struct PointLightShadowSlot {
    pub entity: freecs::Entity,
    pub position: nalgebra_glm::Vec3,
    pub range: f32,
    pub slot_index: u32,
    pub bias: f32,
    pub clean: bool,
}

/// Per-slot persistent shadow cache: which light last rendered into the slot and
/// a content hash of its view-projection plus every occluder that affects it. A
/// slot is reused untouched across frames while both stay equal.
#[derive(Clone, Copy, Default)]
pub struct ShadowSlotCache {
    pub occupant: Option<freecs::Entity>,
    pub hash: u64,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PointLightShadowData {
    pub position: [f32; 3],
    pub range: f32,
    pub bias: f32,
    pub shadow_index: i32,
    pub _padding: [f32; 2],
}

#[repr(C, align(256))]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PointShadowUniforms {
    pub view_projection: [[f32; 4]; 4],
    pub light_position: [f32; 3],
    pub light_range: f32,
    pub _padding: [f32; 44],
}
