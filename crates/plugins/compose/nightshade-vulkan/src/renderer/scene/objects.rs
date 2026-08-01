pub const FLAG_VISIBLE: u32 = 1;
pub const FLAG_SKINNED: u32 = 2;
pub const FLAG_SHADOW_CASTER: u32 = 4;
pub const FLAG_DOUBLE_SIDED: u32 = 8;
pub const FLAG_OVERLAY: u32 = 16;
pub const FLAG_DEBUG_VIEW_EXEMPT: u32 = 32;

pub fn flips_winding(transform: &nalgebra_glm::Mat4) -> bool {
    nalgebra_glm::determinant(&nalgebra_glm::mat4_to_mat3(transform)) < 0.0
}

pub const NO_MATERIAL: u32 = u32::MAX;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuObject {
    pub transform: [[f32; 4]; 4],
    pub previous_transform: [[f32; 4]; 4],
    pub normal_transform: [[f32; 4]; 4],
    pub bounds: [f32; 4],
    pub mesh_slot: u32,
    pub material_index: u32,
    pub culling_mask: u32,
    pub flags: u32,
    pub vertex_offset: u32,
    pub previous_vertex_offset: u32,
    pub entity_id: u32,
    pub run: u32,
    pub posed_meshlets: u32,
    pub morph_weights: [f32; 8],
    pub tint: [f32; 4],
}

pub fn normal_transform(transform: &nalgebra_glm::Mat4) -> [[f32; 4]; 4] {
    let upper = nalgebra_glm::mat4_to_mat3(transform);
    let normal = nalgebra_glm::transpose(&nalgebra_glm::inverse(&upper));
    let mut columns = [[0.0f32; 4]; 4];
    for column in 0..3 {
        for row in 0..3 {
            columns[column][row] = normal[(row, column)];
        }
    }
    columns[3][3] = 1.0;
    columns
}

pub use nightshade_render::queries::projection::matrix_columns;

pub const SOLO_RUN: u32 = 1 << 16;

pub fn run_of(length: u32, position: u32) -> u32 {
    (length << 16) | position
}
