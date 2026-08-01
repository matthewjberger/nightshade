#[derive(Clone, Copy, Debug)]
pub struct PoolWindow {
    pub visible_start: usize,
    pub top_height: f32,
    pub bottom_height: f32,
}

pub fn compute(
    scroll_offset: f32,
    item_height: f32,
    total_items: usize,
    pool_size: usize,
) -> PoolWindow {
    let visible_start = if item_height > 0.0 {
        (scroll_offset / item_height).floor() as usize
    } else {
        0
    };
    let visible_start = visible_start.min(total_items.saturating_sub(pool_size));
    let visible_end = total_items.min(visible_start + pool_size);
    let top_height = visible_start as f32 * item_height;
    let bottom_height = total_items.saturating_sub(visible_end) as f32 * item_height;
    PoolWindow {
        visible_start,
        top_height,
        bottom_height,
    }
}
