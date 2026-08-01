use crate::contract::data::text_data::TextMesh;

#[derive(Clone, Default)]
pub struct RenderText {
    pub mesh: TextMesh,
    pub color: nalgebra_glm::Vec4,
    pub outline_color: nalgebra_glm::Vec4,
    pub outline_width: f32,
    pub smoothing: f32,
    pub billboard: bool,
    pub transform: nalgebra_glm::Mat4,
    pub char_count: usize,
    pub character_colors: Option<Vec<Option<nalgebra_glm::Vec4>>>,
}
