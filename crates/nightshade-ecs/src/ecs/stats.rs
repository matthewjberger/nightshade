use crate::Mask;

#[derive(Clone, Debug, Default)]
pub struct WorldStats {
    pub entity_count: usize,
    pub table_count: usize,
    pub empty_table_count: usize,
    pub largest_table_rows: usize,
    pub table_rows: Vec<(Mask, usize)>,
    pub component_count: usize,
    pub tag_count: usize,
    pub remaining_mask_bits: u32,
    pub structural_log_entries: usize,
    pub query_cache_entries: usize,
    pub resource_count: usize,
    pub event_channels: usize,
    pub pending_commands: usize,
}
