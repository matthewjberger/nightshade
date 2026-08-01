use super::RenderView;

pub const CLUSTER_GRID_X: u32 = 16;
pub const CLUSTER_GRID_Y: u32 = 9;
pub const CLUSTER_GRID_Z: u32 = 24;
pub const CLUSTER_COUNT: u32 = CLUSTER_GRID_X * CLUSTER_GRID_Y * CLUSTER_GRID_Z;
pub const MAX_LIGHTS_PER_CLUSTER: u32 = 256;

pub const CLUSTER_SLICE_NEAR: f32 = 0.5;

pub fn cluster_slice_near(camera_z_near: f32) -> f32 {
    camera_z_near.max(CLUSTER_SLICE_NEAR)
}

pub fn cluster_slicing(view: &RenderView) -> [f32; 4] {
    let near = view.z_near.max(1.0e-3);
    let far = view.z_far.max(near * 2.0);
    let span = (far / near).ln();
    let scale = CLUSTER_GRID_Z as f32 / span;
    [scale, -near.ln() * scale, near, far]
}
