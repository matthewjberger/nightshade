pub(super) use super::super::projection::LightData;

/// The image-based lighting texture views bound for a frame, holding two probes
/// the shader cross-fades between by `blend_factor`.
pub struct BlendedIblViews {
    /// Precomputed split-sum BRDF lookup table.
    pub brdf_lut: wgpu::TextureView,
    /// Diffuse irradiance cubemap of the first probe.
    pub irradiance_a: wgpu::TextureView,
    /// Prefiltered specular cubemap of the first probe.
    pub prefiltered_a: wgpu::TextureView,
    /// Diffuse irradiance cubemap of the second probe.
    pub irradiance_b: wgpu::TextureView,
    /// Prefiltered specular cubemap of the second probe.
    pub prefiltered_b: wgpu::TextureView,
    /// Cross-fade weight from the first probe to the second, in 0 to 1.
    pub blend_factor: f32,
}

pub use super::super::cluster::{
    CLUSTER_GRID_X, CLUSTER_GRID_Y, CLUSTER_GRID_Z, ClusterBounds, ClusterUniforms, LightGrid,
    MAX_LIGHTS_PER_CLUSTER, TOTAL_CLUSTERS, cluster_slice_near,
};
pub(super) use super::super::projection::{AreaLightData, AreaUniforms, MAX_AREA_LIGHTS};

/// Maximum joint matrices a single skin may reference.
pub(super) const MAX_JOINTS_PER_SKIN: usize = 256;
/// Initial skin capacity before the buffers grow.
pub(super) const INITIAL_MAX_SKINS: usize = 100;
/// Initial joint-matrix capacity across all skins before the buffers grow.
pub(super) const INITIAL_MAX_TOTAL_BONES: usize = 5_000;
/// Initial skinned vertex capacity before the buffer grows.
pub(super) const INITIAL_SKINNED_VERTEX_COUNT: usize = 100_000;
/// Initial skinned index capacity before the buffer grows.
pub(super) const INITIAL_SKINNED_INDEX_COUNT: usize = 300_000;
/// Initial morph displacement capacity before the buffer grows.
pub(super) const INITIAL_MORPH_DISPLACEMENT_COUNT: usize = 100_000;

#[repr(C, align(256))]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
/// Per-frame uniform block for the skinned mesh pass, `std140`-aligned to 256
/// bytes.
pub(super) struct SkinnedMeshUniforms {
    /// Camera view matrix.
    pub(super) view: [[f32; 4]; 4],
    /// Camera projection matrix.
    pub(super) projection: [[f32; 4]; 4],
    /// Camera world position, w unused.
    pub(super) camera_position: [f32; 4],
    /// Active light count in x, remaining lanes padding.
    pub(super) num_lights: [u32; 4],
    /// Ambient light color, w unused.
    pub(super) ambient_light: [f32; 4],
    /// Primary directional light view-projection for shadow sampling.
    pub(super) light_view_projection: [[f32; 4]; 4],
    /// Depth-comparison bias for shadow sampling.
    pub(super) shadow_bias: f32,
    /// Nonzero enables shadow sampling.
    pub(super) shadows_enabled: f32,
    /// Nonzero forces unlit shading for every object.
    pub(super) global_unlit: f32,
    /// Normal-offset bias for shadow sampling.
    pub(super) shadow_normal_bias: f32,
    /// Vertex-snap grid resolution, in pixels.
    pub(super) snap_resolution: [f32; 2],
    /// Nonzero enables retro vertex snapping.
    pub(super) snap_enabled: u32,
    /// Nonzero enables affine (perspective-incorrect) texture mapping.
    pub(super) affine_enabled: u32,
    /// Fog color, linear RGB.
    pub(super) fog_color: [f32; 3],
    /// Nonzero enables distance fog.
    pub(super) fog_enabled: u32,
    /// Fog start distance, in world units.
    pub(super) fog_start: f32,
    /// Fog end distance, in world units.
    pub(super) fog_end: f32,
    /// Number of active shadow cascades.
    pub(super) cascade_count: u32,
    /// Padding for 16-byte alignment.
    pub(super) _padding2: f32,
    /// Per-cascade view-projection matrices.
    pub(super) cascade_view_projections: [[[f32; 4]; 4]; crate::wgpu::passes::NUM_SHADOW_CASCADES],
    /// Per-cascade far split distances, in world units.
    pub(super) cascade_split_distances: [f32; crate::wgpu::passes::NUM_SHADOW_CASCADES],
    /// Per-cascade atlas offsets, xy used.
    pub(super) cascade_atlas_offsets: [[f32; 4]; crate::wgpu::passes::NUM_SHADOW_CASCADES],
    /// Per-cascade atlas scale.
    pub(super) cascade_atlas_scale: [f32; 4],
    /// Elapsed time, in seconds.
    pub(super) time: f32,
    /// PBR debug visualization mode.
    pub(super) pbr_debug_mode: u32,
    /// Nonzero draws texture debug stripes.
    pub(super) texture_debug_stripes: u32,
    /// Animation speed of the texture debug stripes.
    pub(super) texture_debug_stripes_speed: f32,
    /// Primary directional light direction, w unused.
    pub(super) directional_light_direction: [f32; 4],
    /// Cross-fade weight between the two IBL probes.
    pub(super) ibl_blend_factor: f32,
    /// Depth scale for order-independent transparency weighting.
    pub(super) oit_z_scale: f32,
    /// Padding before `flat_color`.
    pub(super) _pad_pre_flat: [f32; 2],
    /// Flat debug fill color, RGBA.
    pub(super) flat_color: [f32; 4],
    /// Previous-frame view-projection for motion vectors.
    pub(super) prev_view_projection: [[f32; 4]; 4],
    /// Current-frame view-projection for motion vectors.
    pub(super) cur_view_projection: [[f32; 4]; 4],
    /// Padding to the 256-byte alignment.
    pub(super) _padding3: [f32; 44],
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
/// Per-object record for one skinned draw, indexed by the skinning and shading
/// shaders.
pub(super) struct SkinnedObjectData {
    /// Index into the transform buffer.
    pub(super) transform_index: u32,
    /// Mesh index in the skinned mesh table.
    pub(super) mesh_id: u32,
    /// Material index.
    pub(super) material_id: u32,
    /// Base offset into the joint-matrix buffer.
    pub(super) joint_offset: u32,
    /// Morph-target blend weights.
    pub(super) morph_weights: [f32; 8],
    /// Number of active morph targets.
    pub(super) morph_target_count: u32,
    /// Base offset into the morph displacement buffer.
    pub(super) morph_displacement_offset: u32,
    /// Base vertex offset in the skinned vertex buffer.
    pub(super) mesh_vertex_offset: u32,
    /// Vertex count for this object.
    pub(super) mesh_vertex_count: u32,
    /// Nonzero flips triangle winding.
    pub(super) flip_winding: u32,
    /// Source entity id, for picking.
    pub(super) entity_id: u32,
    /// Nonzero marks the object transparent.
    pub(super) is_transparent: u32,
    /// Padding for 16-byte alignment.
    pub(super) _padding: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
/// One vertex's per-target displacement, applied by the skinning shader scaled by
/// the morph weight.
pub(super) struct MorphDisplacement {
    /// Position delta.
    pub(super) position: [f32; 3],
    /// Padding for 16-byte alignment.
    pub(super) _pad0: f32,
    /// Normal delta.
    pub(super) normal: [f32; 3],
    /// Padding for 16-byte alignment.
    pub(super) _pad1: f32,
    /// Tangent delta.
    pub(super) tangent: [f32; 3],
    /// Padding for 16-byte alignment.
    pub(super) _pad2: f32,
}

pub use super::super::material_gpu::MaterialData;

#[derive(Debug, Clone)]
/// CPU-side allocation record for one skinned mesh within the shared buffers.
pub(super) struct SkinnedMeshData {
    /// Base vertex offset in the skinned vertex buffer.
    pub(super) vertex_offset: u32,
    /// Vertex count.
    pub(super) vertex_count: u32,
    /// Base index offset in the skinned index buffer.
    pub(super) index_offset: u32,
    /// Index count.
    pub(super) index_count: u32,
    /// Skin index this mesh binds to.
    pub(super) _skin_index: u32,
    /// Base offset into the morph displacement buffer.
    pub(super) morph_displacement_offset: u32,
    /// Number of morph targets.
    pub(super) morph_target_count: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
/// Per-object input to the GPU cull compute pass for skinned draws.
pub(super) struct SkinnedCullObject {
    /// Bounding sphere, xyz center and w radius, in world space.
    pub(super) bounds: [f32; 4],
    /// Index of the indirect draw command this object feeds.
    pub(super) command_index: u32,
    /// Padding for 16-byte alignment.
    pub(super) _pad: [u32; 3],
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
/// Per-object input to the compute pass that derives skinned bounds from posed
/// bones.
pub(super) struct SkinnedBoundsInput {
    /// Base offset into the joint-matrix buffer.
    pub(super) base_bone: u32,
    /// Number of joints in the skin.
    pub(super) joint_count: u32,
    /// Rest-pose bounding radius, in world units.
    pub(super) rest_radius: f32,
    /// Padding for 16-byte alignment.
    pub(super) pad: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
/// Indexed indirect draw arguments, matching the GPU indirect-draw layout.
pub(super) struct SkinnedDrawIndexedIndirect {
    /// Number of indices to draw.
    pub(super) index_count: u32,
    /// Number of instances to draw.
    pub(super) instance_count: u32,
    /// First index in the index buffer.
    pub(super) first_index: u32,
    /// Value added to each index before fetching a vertex.
    pub(super) base_vertex: i32,
    /// First instance id.
    pub(super) first_instance: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
/// Uniform block for the skinned cull compute pass.
pub(super) struct SkinnedCullUniforms {
    /// Camera frustum planes tested against each object's bounds.
    pub(super) frustum_planes: [[f32; 4]; 6],
    /// Number of cull objects to process.
    pub(super) object_count: u32,
    /// Padding for 16-byte alignment.
    pub(super) _pad: [u32; 3],
}
