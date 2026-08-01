#[derive(Default, Clone)]
pub struct CleanupState {
    pub mesh_cursor: usize,
    pub material_cursor: usize,
    pub sweep_frame: u64,
    pub owner_frame: u64,
}
