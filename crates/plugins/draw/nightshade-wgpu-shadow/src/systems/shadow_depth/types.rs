pub use nightshade_render::data::shadow::CASCADE_SPLIT_DISTANCES;
pub use nightshade_render::data::shadow::SpotlightShadowData;
pub use nightshade_wgpu::backend::caps::{
    CASCADE_ATLAS_SIZE, CASCADE_ATLAS_SLOTS_PER_ROW, CASCADE_SLOT_RESOLUTION,
    MAX_POINT_LIGHT_SHADOWS, MAX_SPOTLIGHT_SHADOWS, POINT_SHADOW_FACE_SIZE, POINT_SHADOW_NUM_FACES,
    SPOTLIGHT_ATLAS_SIZE, SPOTLIGHT_ATLAS_SLOTS_PER_ROW, point_shadow_layers, point_shadow_slots,
};
pub use nightshade_wgpu_shading::backend::light::PointLightShadowData;

pub const MAX_JOINTS_PER_SKIN: usize = 256;
pub const INITIAL_MAX_SKINS: usize = 100;
pub const INITIAL_MAX_TOTAL_BONES: usize = 5_000;

pub use nightshade_render::data::light_gpu::NUM_SHADOW_CASCADES;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ShadowVertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
    pub tex_coords_1: [f32; 2],
    pub alpha: f32,
}

impl From<&nightshade_render::data::geometry::Vertex> for ShadowVertex {
    fn from(vertex: &nightshade_render::data::geometry::Vertex) -> Self {
        Self {
            position: vertex.position,
            tex_coords: vertex.tex_coords,
            tex_coords_1: vertex.tex_coords_1,
            alpha: vertex.color[3],
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ShadowOccluder {
    pub transform_index: u32,
    pub mesh_geo_id: u32,
    pub batch_id: u32,
    pub material_index: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ShadowMeshGeo {
    pub index_count: u32,
    pub first_index: u32,
    pub base_vertex: i32,
    pub padding: u32,
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

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ShadowCullUniforms {
    pub frustum_planes: [[f32; 4]; 6],
    pub occluder_count: u32,
    pub indirect_offset: u32,
    pub padding_0: u32,
    pub padding_1: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SkinnedShadowObjectData {
    pub index_offset: u32,
    pub index_count: u32,
    pub vertex_offset: u32,
    pub joint_offset: u32,
    pub material_index: u32,
    pub padding: [u32; 3],
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SkinnedShadowCullObject {
    pub bounds: [f32; 4],
    pub command_index: u32,
    pub padding: [u32; 3],
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ShadowUniforms {
    pub light_view_projection: [[f32; 4]; 4],
    pub time: f32,
    pub padding_time: [f32; 3],
    pub padding: [[f32; 4]; 4],
    pub padding_2: [[f32; 4]; 4],
    pub padding_3: [[f32; 4]; 4],
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CascadeData {
    pub view_projections: [[[f32; 4]; 4]; NUM_SHADOW_CASCADES],
    pub split_distances: [f32; NUM_SHADOW_CASCADES],
    pub atlas_offsets: [[f32; 2]; NUM_SHADOW_CASCADES],
    pub atlas_scale: [f32; 2],
    pub padding: [f32; 2],
}

#[derive(Debug, Clone)]
pub struct SpotlightShadowSlot {
    pub view_projection: nalgebra_glm::Mat4,
    pub slot_index: u32,
    pub atlas_x_pixels: u32,
    pub atlas_y_pixels: u32,
    pub atlas_size_pixels: u32,
    pub clean: bool,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct PointLightShadowSlot {
    pub position: nalgebra_glm::Vec3,
    pub range: f32,
    pub slot_index: u32,
    pub clean: bool,
}

#[derive(Clone, Copy, Default)]
pub struct ShadowSlotCache {
    pub occupant: Option<nightshade::ecs::Entity>,
    pub hash: u64,
}

#[repr(C, align(256))]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PointShadowUniforms {
    pub view_projection: [[f32; 4]; 4],
    pub light_position: [f32; 3],
    pub light_range: f32,
    pub time: f32,
    pub tail_to_block: [f32; 43],
}

pub const NO_SHADOW_MATERIAL: u32 = u32::MAX;

#[derive(Default)]
pub struct ShadowFrameChanges {
    pub added: Vec<nightshade::ecs::Entity>,
    pub removed: Vec<nightshade::ecs::Entity>,
    pub moved: Vec<nightshade::ecs::Entity>,
    pub state_dirty: Vec<nightshade::ecs::Entity>,
    pub material_dirty: Vec<nightshade::ecs::Entity>,
}

pub enum OccluderUpload {
    All,
    Touched { slots: Vec<u32>, set_changed: bool },
}
