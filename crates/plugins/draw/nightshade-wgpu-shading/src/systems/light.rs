#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PointLightShadowData {
    pub position: [f32; 3],
    pub range: f32,
    pub bias: f32,
    pub shadow_index: i32,
    pub padding: [f32; 2],
}
