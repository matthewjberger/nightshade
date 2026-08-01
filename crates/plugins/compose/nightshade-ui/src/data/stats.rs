use nalgebra_glm::Vec4;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StatPanel {
    pub margin: f32,
    pub width: f32,
    pub label_width: f32,
    pub row_height: f32,
    pub font_size: f32,
    pub header_size: f32,
    pub label_color: Vec4,
    pub value_color: Vec4,
    pub header_color: Vec4,
}

impl Default for StatPanel {
    fn default() -> Self {
        Self {
            margin: 12.0,
            width: 240.0,
            label_width: 120.0,
            row_height: 16.0,
            font_size: 12.0,
            header_size: 11.0,
            label_color: Vec4::new(0.60, 0.62, 0.68, 1.0),
            value_color: Vec4::new(0.95, 0.72, 0.35, 1.0),
            header_color: Vec4::new(0.45, 0.48, 0.55, 1.0),
        }
    }
}
