use nalgebra_glm::Vec4;

#[derive(Clone, Copy, Debug)]
pub struct MenuStyle {
    pub background: Vec4,
    pub border: Vec4,
    pub corner_radius: f32,
    pub border_width: f32,
    pub padding: f32,
    pub spacing: f32,
    pub visible: bool,
    pub heading_size: f32,
    pub heading_color: Vec4,
    pub centered: bool,
    pub row_size: f32,
    pub row_color: Vec4,
    pub outline: Vec4,
    pub outline_width: f32,
}

impl Default for MenuStyle {
    fn default() -> Self {
        Self {
            background: Vec4::new(0.05, 0.05, 0.08, 0.88),
            border: Vec4::new(1.0, 1.0, 1.0, 0.12),
            corner_radius: 10.0,
            border_width: 1.0,
            padding: 16.0,
            spacing: 10.0,
            visible: false,
            heading_size: 30.0,
            heading_color: Vec4::new(1.0, 1.0, 1.0, 1.0),
            centered: false,
            row_size: 15.0,
            row_color: Vec4::new(0.72, 0.75, 0.82, 1.0),
            outline: Vec4::new(0.0, 0.0, 0.0, 0.0),
            outline_width: 0.0,
        }
    }
}

pub fn dark_menu_style() -> MenuStyle {
    MenuStyle {
        background: Vec4::new(0.04, 0.05, 0.07, 0.92),
        border: Vec4::new(1.0, 1.0, 1.0, 0.14),
        padding: 14.0,
        spacing: 18.0,
        heading_color: Vec4::new(0.86, 0.88, 0.94, 1.0),
        row_size: 15.0,
        row_color: Vec4::new(0.62, 0.66, 0.74, 1.0),
        ..Default::default()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CardStyle {
    pub card_background: Vec4,
    pub card_border: Vec4,
    pub card_radius: f32,
    pub card_border_width: f32,
    pub frame_corner: f32,
    pub frame_tint: Vec4,
    pub title_height: f32,
    pub title_color: Vec4,
    pub heading_size: f32,
    pub heading_color: Vec4,
    pub footer_height: f32,
    pub footer_color: Vec4,
    pub body_padding: f32,
    pub row_gap: f32,
    pub row_height: f32,
    pub row_padding: f32,
    pub row_spacing: f32,
    pub row_color: Vec4,
    pub row_alt_color: Vec4,
    pub small_size: f32,
    pub body_size: f32,
    pub text_color: Vec4,
    pub bright_color: Vec4,
    pub dim_color: Vec4,
    pub action_width: f32,
    pub action_height: f32,
    pub bar_height: f32,
    pub bar_border: Vec4,
    pub bar_background: Vec4,
    pub cell_size: f32,
    pub cell_icon: f32,
    pub cell_gap: f32,
    pub cell_color: Vec4,
    pub cell_border: Vec4,
    pub count_size: f32,
}

impl Default for CardStyle {
    fn default() -> Self {
        Self {
            card_background: Vec4::new(0.07, 0.06, 0.09, 0.96),
            card_border: Vec4::new(0.85, 0.7, 0.45, 0.9),
            card_radius: 12.0,
            card_border_width: 2.0,
            frame_corner: 16.0,
            frame_tint: Vec4::new(1.0, 1.0, 1.0, 1.0),
            title_height: 48.0,
            title_color: Vec4::new(0.16, 0.13, 0.09, 1.0),
            heading_size: 24.0,
            heading_color: Vec4::new(1.0, 0.82, 0.45, 1.0),
            footer_height: 58.0,
            footer_color: Vec4::new(0.11, 0.1, 0.13, 1.0),
            body_padding: 16.0,
            row_gap: 4.0,
            row_height: 38.0,
            row_padding: 8.0,
            row_spacing: 8.0,
            row_color: Vec4::new(0.15, 0.14, 0.18, 0.75),
            row_alt_color: Vec4::new(0.11, 0.1, 0.14, 0.75),
            small_size: 15.0,
            body_size: 18.0,
            text_color: Vec4::new(1.0, 1.0, 1.0, 1.0),
            bright_color: Vec4::new(1.0, 1.0, 1.0, 1.0),
            dim_color: Vec4::new(0.7, 0.66, 0.6, 1.0),
            action_width: 88.0,
            action_height: 30.0,
            bar_height: 22.0,
            bar_border: Vec4::new(0.0, 0.0, 0.0, 0.85),
            bar_background: Vec4::new(0.05, 0.05, 0.07, 0.92),
            cell_size: 58.0,
            cell_icon: 36.0,
            cell_gap: 8.0,
            cell_color: Vec4::new(0.13, 0.12, 0.16, 0.95),
            cell_border: Vec4::new(0.42, 0.4, 0.46, 0.95),
            count_size: 14.0,
        }
    }
}
