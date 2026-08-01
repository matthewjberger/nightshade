use nightshade::ecs::Entity;

use crate::data::light_gpu::NUM_SHADOW_CASCADES;

pub const CASCADE_SPLIT_DISTANCES: [f32; NUM_SHADOW_CASCADES] = [10.0, 40.0, 90.0, 150.0];
pub const CASCADE_REFERENCE_NEAR: f32 = 0.01;

pub struct SpotlightAtlasSlot {
    pub entity: Entity,
    pub view_projection: nalgebra_glm::Mat4,
    pub atlas_x_pixels: u32,
    pub atlas_y_pixels: u32,
    pub atlas_size_pixels: u32,
    pub slot_index: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SpotlightShadowData {
    pub view_projection: [[f32; 4]; 4],
    pub atlas_offset: [f32; 2],
    pub atlas_scale: [f32; 2],
    pub bias: f32,
    pub padding: [f32; 3],
}
