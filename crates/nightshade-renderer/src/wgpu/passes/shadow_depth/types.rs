//! GPU and CPU data types for the shadow depth pass: cascades, spotlight and
//! point-light slots, occluders, and cull uniforms.

/// Maximum joint matrices a single skin can address.
pub const MAX_JOINTS_PER_SKIN: usize = 256;
/// Initial skin capacity of the shadow skinning buffers.
pub const INITIAL_MAX_SKINS: usize = 100;
/// Initial bone-transform capacity across all skins.
pub const INITIAL_MAX_TOTAL_BONES: usize = 5_000;
/// Maximum spotlight shadow slots in the atlas.
pub const MAX_SPOTLIGHT_SHADOWS: usize = 64;
/// Spotlight shadow atlas slots per row.
pub const SPOTLIGHT_ATLAS_SLOTS_PER_ROW: u32 = 8;

/// Number of directional-light shadow cascades.
pub const NUM_SHADOW_CASCADES: usize = 4;
/// Cascade shadow atlas slots per row.
pub const CASCADE_ATLAS_SLOTS_PER_ROW: u32 = 2;
/// Cascade shadow atlas edge length in texels, `4096` on wasm and `8192`
/// natively.
pub const CASCADE_ATLAS_SIZE: u32 = if cfg!(target_arch = "wasm32") {
    4096
} else {
    8192
};
/// Edge length in texels of one cascade atlas slot.
pub const CASCADE_SLOT_RESOLUTION: f32 = (CASCADE_ATLAS_SIZE / CASCADE_ATLAS_SLOTS_PER_ROW) as f32;
/// Far distance of each cascade in world units. The near boundaries (10, 40)
/// match the previous close-range tuning so zoomed-in shadows keep their density,
/// while the far reach is pulled in from 500 to 150 and the freed cascade is
/// redistributed into the mid range. This concentrates texel density where an
/// orbit camera actually frames its subject, so shadows stay crisp when the
/// camera pulls back instead of collapsing into the single 150..500 slab.
pub const CASCADE_SPLIT_DISTANCES: [f32; NUM_SHADOW_CASCADES] = [10.0, 40.0, 90.0, 150.0];

/// Reference camera near plane the base cascade splits are tuned for.
pub const CASCADE_REFERENCE_NEAR: f32 = 0.01;

/// Scales the cascade far distances for a camera whose near plane is closer than
/// `CASCADE_REFERENCE_NEAR`, never growing them past the base splits. Returns the
/// per-cascade far distances in world units.
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

/// Maximum point-light shadow slots in the cubemap array.
pub const MAX_POINT_LIGHT_SHADOWS: usize = 16;
/// Edge length in texels of one point-shadow cubemap face.
pub const POINT_SHADOW_FACE_SIZE: u32 = 256;
/// Cubemap faces per point-light shadow.
pub const POINT_SHADOW_NUM_FACES: usize = 6;

/// One shadow-casting occluder: which transform it uses, which mesh geometry
/// it draws, and which indirect batch it belongs to. Rebuilt each frame from
/// the shadow caster list. The GPU cull writes `transform_index` into the
/// visible-index buffer for each view the occluder survives.
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ShadowOccluder {
    /// Index into the shadow transform buffer.
    pub transform_index: u32,
    /// Index into the shadow mesh-geometry table.
    pub mesh_geo_id: u32,
    /// Indirect batch this occluder draws in.
    pub batch_id: u32,
    /// Padding for 16-byte alignment.
    pub _pad: u32,
}

/// Draw parameters for one registered static shadow mesh: its index range and
/// base vertex within the shared shadow buffers.
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ShadowMeshGeo {
    /// Number of indices in the mesh.
    pub index_count: u32,
    /// First index into the shared shadow index buffer.
    pub first_index: u32,
    /// Vertex offset added to each index.
    pub base_vertex: i32,
    /// Padding for 16-byte alignment.
    pub _pad: u32,
}

/// Bounding sphere of a shadow mesh.
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ShadowMeshBounds {
    /// Sphere center.
    pub center: [f32; 3],
    /// Sphere radius.
    pub radius: f32,
}

/// Indexed indirect draw command for shadow batches, matching wgpu's
/// `draw_indexed_indirect` argument layout.
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ShadowDrawIndexedIndirect {
    /// Number of indices to draw.
    pub index_count: u32,
    /// Number of instances to draw.
    pub instance_count: u32,
    /// First index into the shared shadow index buffer.
    pub first_index: u32,
    /// Vertex offset added to each index.
    pub base_vertex: i32,
    /// First instance index, carrying the visible-index offset for the batch.
    pub first_instance: u32,
}

/// Per-view culling parameters. `indirect_offset` selects the view's run of
/// per-batch indirect commands; the visible-index offsets are baked into each
/// command's `first_instance` so the cull only needs the frustum and counts.
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ShadowCullUniforms {
    /// Six frustum planes of the shadow view, each packed as `vec4` normal and
    /// distance.
    pub frustum_planes: [[f32; 4]; 6],
    /// Number of occluders to test.
    pub occluder_count: u32,
    /// Offset selecting this view's run of per-batch indirect commands.
    pub indirect_offset: u32,
    /// Padding for 16-byte alignment.
    pub _pad0: u32,
    /// Padding for 16-byte alignment.
    pub _pad1: u32,
}

/// Per-instance data for a skinned shadow occluder.
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SkinnedShadowObjectData {
    /// Index into the shadow transform buffer.
    pub transform_index: u32,
    /// Skinned shadow mesh index.
    pub mesh_id: u32,
    /// Material index.
    pub material_id: u32,
    /// First joint matrix for this instance in the joint buffer.
    pub joint_offset: u32,
}

/// Cull input for one skinned shadow occluder.
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SkinnedShadowCullObject {
    /// Bounding sphere as `[center_x, center_y, center_z, radius]`.
    pub bounds: [f32; 4],
    /// Indirect command this object contributes to.
    pub command_index: u32,
    /// Padding for 16-byte alignment.
    pub _pad: [u32; 3],
}

/// Uniform block for a single shadow view.
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ShadowUniforms {
    /// World-to-light clip-space matrix.
    pub light_view_projection: [[f32; 4]; 4],
    /// Reserved padding.
    pub _padding: [[f32; 4]; 4],
    /// Reserved padding.
    pub _padding2: [[f32; 4]; 4],
    /// Reserved padding.
    pub _padding3: [[f32; 4]; 4],
    /// Reserved padding.
    pub _padding4: [f32; 4],
}

/// Directional-light cascade set: per-cascade matrices, split distances, and
/// atlas placement.
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CascadeData {
    /// World-to-clip matrix per cascade.
    pub view_projections: [[[f32; 4]; 4]; NUM_SHADOW_CASCADES],
    /// Far distance of each cascade in world units.
    pub split_distances: [f32; NUM_SHADOW_CASCADES],
    /// Normalized atlas offset of each cascade slot.
    pub atlas_offsets: [[f32; 2]; NUM_SHADOW_CASCADES],
    /// Normalized size of one cascade atlas slot.
    pub atlas_scale: [f32; 2],
    /// Padding for 16-byte alignment.
    pub _padding: [f32; 2],
}

/// CPU-side record of a spotlight occupying one shadow atlas slot.
#[derive(Debug, Clone)]
pub struct SpotlightShadowSlot {
    /// Light entity assigned to the slot.
    pub entity: nightshade_ecs::Entity,
    /// World-to-light clip-space matrix.
    pub view_projection: nalgebra_glm::Mat4,
    /// Atlas slot index.
    pub slot_index: u32,
    /// Depth-compare bias.
    pub bias: f32,
    /// Slot x origin in atlas texels.
    pub atlas_x_pixels: u32,
    /// Slot y origin in atlas texels.
    pub atlas_y_pixels: u32,
    /// Slot edge length in atlas texels.
    pub atlas_size_pixels: u32,
    /// Whether the slot's cached contents are still valid this frame.
    pub clean: bool,
}

/// GPU shadow parameters for one spotlight.
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SpotlightShadowData {
    /// World-to-light clip-space matrix.
    pub view_projection: [[f32; 4]; 4],
    /// Normalized atlas offset of the slot.
    pub atlas_offset: [f32; 2],
    /// Normalized size of the slot.
    pub atlas_scale: [f32; 2],
    /// Depth-compare bias.
    pub bias: f32,
    /// Padding for 16-byte alignment.
    pub _padding: [f32; 3],
}

/// CPU-side record of a point light occupying one cubemap shadow slot.
#[derive(Debug, Clone)]
pub struct PointLightShadowSlot {
    /// Light entity assigned to the slot.
    pub entity: nightshade_ecs::Entity,
    /// Light world position.
    pub position: nalgebra_glm::Vec3,
    /// Light range in world units.
    pub range: f32,
    /// Cubemap array slot index.
    pub slot_index: u32,
    /// Depth-compare bias.
    pub bias: f32,
    /// Whether the slot's cached contents are still valid this frame.
    pub clean: bool,
}

/// Per-slot persistent shadow cache: which light last rendered into the slot and
/// a content hash of its view-projection plus every occluder that affects it. A
/// slot is reused untouched across frames while both stay equal.
#[derive(Clone, Copy, Default)]
pub struct ShadowSlotCache {
    /// Light that last rendered into the slot, if any.
    pub occupant: Option<nightshade_ecs::Entity>,
    /// Content hash of the occupant's view-projection and affecting occluders.
    pub hash: u64,
}

/// GPU shadow parameters for one point light.
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PointLightShadowData {
    /// Light world position.
    pub position: [f32; 3],
    /// Light range in world units.
    pub range: f32,
    /// Depth-compare bias.
    pub bias: f32,
    /// Cubemap array slot index, or `-1` when the light casts no shadow.
    pub shadow_index: i32,
    /// Padding for 16-byte alignment.
    pub _padding: [f32; 2],
}

/// Per-face uniform block for point-shadow rendering, aligned to the 256-byte
/// dynamic-offset stride.
#[repr(C, align(256))]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PointShadowUniforms {
    /// World-to-clip matrix for the cubemap face.
    pub view_projection: [[f32; 4]; 4],
    /// Light world position.
    pub light_position: [f32; 3],
    /// Light range in world units.
    pub light_range: f32,
    /// Padding to the 256-byte uniform stride.
    pub _padding: [f32; 44],
}
