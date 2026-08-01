use nightshade::prelude::*;

pub const PANEL_BG: Vec4 = Vec4::new(0.05, 0.06, 0.09, 0.92);
pub const PANEL_BORDER: Vec4 = Vec4::new(0.14, 0.17, 0.24, 0.7);
pub const SECTION_HEADER: Vec4 = Vec4::new(0.95, 0.55, 0.30, 1.0);
pub const TEXT_PRIMARY: Vec4 = Vec4::new(0.92, 0.94, 1.0, 1.0);
pub const TEXT_DIM: Vec4 = Vec4::new(0.60, 0.66, 0.76, 1.0);
pub const ACCENT_HOT: Vec4 = Vec4::new(1.0, 0.82, 0.55, 1.0);

pub const PANEL_WIDTH: f32 = 352.0;
pub const PANEL_HEIGHT: f32 = 480.0;

pub const MARGIN: f32 = 18.0;
pub const LABEL_WIDTH: f32 = 124.0;
pub const ROW_HEIGHT: f32 = 24.0;
pub const BUTTON_HEIGHT: f32 = 28.0;
pub const FONT_SIZE: f32 = 13.0;

pub fn stat_panel() -> nightshade_ui::data::stats::StatPanel {
    nightshade_ui::data::stats::StatPanel {
        margin: MARGIN,
        width: PANEL_WIDTH,
        label_width: LABEL_WIDTH,
        row_height: ROW_HEIGHT,
        font_size: FONT_SIZE,
        header_size: 11.0,
        label_color: TEXT_DIM,
        value_color: ACCENT_HOT,
        header_color: SECTION_HEADER,
    }
}
