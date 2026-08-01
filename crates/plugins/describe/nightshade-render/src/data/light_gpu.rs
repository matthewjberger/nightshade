pub const NUM_SHADOW_CASCADES: usize = 4;

pub const COOKIE_LAYER_NONE: u32 = 0xFFFFFFFF;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Default)]
pub struct LightData {
    pub position: [f32; 4],
    pub direction: [f32; 4],
    pub color: [f32; 4],
    pub light_type: u32,
    pub range: f32,
    pub inner_cone: f32,
    pub outer_cone: f32,
    pub shadow_index: i32,
    pub light_size: f32,
    pub cookie_layer: u32,
    pub casts_shadow: f32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Default)]
pub struct AreaLightData {
    pub position: [f32; 4],
    pub right: [f32; 4],
    pub up: [f32; 4],
    pub color: [f32; 4],
    pub shape: u32,
    pub range: f32,
    pub radius: f32,
    pub two_sided: u32,
    pub shadow_index: i32,
    pub emissive_layer: u32,
    pub _pad0: f32,
    pub _pad1: f32,
}
