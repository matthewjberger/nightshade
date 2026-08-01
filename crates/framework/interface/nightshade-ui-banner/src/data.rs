use nalgebra_glm::Vec4;

#[derive(Clone, Copy, Debug)]
pub struct BannerStyle {
    pub width: f32,
    pub height: f32,
    pub margin: f32,
    pub size: f32,
    pub color: Vec4,
    pub background: Vec4,
    pub border: Vec4,
    pub corner_radius: f32,
    pub border_width: f32,
    pub outline: Vec4,
    pub outline_width: f32,
}

impl Default for BannerStyle {
    fn default() -> Self {
        Self {
            width: 960.0,
            height: 96.0,
            margin: 16.0,
            size: 20.0,
            color: Vec4::new(0.92, 0.94, 1.0, 1.0),
            background: Vec4::new(0.0, 0.0, 0.0, 0.0),
            border: Vec4::new(0.0, 0.0, 0.0, 0.0),
            corner_radius: 0.0,
            border_width: 0.0,
            outline: Vec4::new(0.0, 0.0, 0.0, 1.0),
            outline_width: 1.0,
        }
    }
}
