use crate::data::catalog::ALL_CATEGORIES;
use crate::resources::Draft;

pub fn toggle_category(draft: &mut Draft, index: usize) {
    draft.enabled ^= 1u32 << index;
}

pub fn set_all_categories(draft: &mut Draft, on: bool) {
    draft.enabled = if on { ALL_CATEGORIES } else { 0 };
}
