use nightshade_platform::prelude::Window;

use crate::resources::Layout;
use crate::theme::{
    ANIMATION_BAR_WIDTH, LAYER_NAME_WIDTH, NARROW_WIDTH, PANEL_MARGIN, PANEL_WIDTH, TOP_BAR_HEIGHT,
    TOP_BAR_HEIGHT_NARROW,
};

pub fn layout_of(width: f32) -> Layout {
    let narrow = width < NARROW_WIDTH;
    let available = (width - PANEL_MARGIN * 2.0).max(200.0);
    Layout {
        narrow,
        top_bar_height: if narrow {
            TOP_BAR_HEIGHT_NARROW
        } else {
            TOP_BAR_HEIGHT
        },
        panel_width: if narrow { available } else { PANEL_WIDTH },
        animation_bar_width: ANIMATION_BAR_WIDTH.min(available),
    }
}

pub fn panel_top(layout: Layout) -> f32 {
    layout.top_bar_height + PANEL_MARGIN
}

pub fn layer_row_width(layout: Layout) -> f32 {
    layout.animation_bar_width - 24.0
}

pub fn layer_weight_width(layout: Layout) -> f32 {
    (layer_row_width(layout) - LAYER_NAME_WIDTH - 114.0).max(80.0)
}

pub fn layout_width(window: &Window) -> f32 {
    let physical = window
        .content_rect
        .filter(|rect| rect.width > 0.5)
        .map(|rect| rect.width)
        .or_else(|| window.cached_viewport_size.map(|(width, _)| width as f32))
        .unwrap_or(1280.0);
    physical / window.cached_scale_factor.max(0.1)
}
