use nalgebra_glm;
use nalgebra_glm::Vec4;
use nightshade_text::contract::data::text_data::TextMesh;
use std::collections::HashMap;

#[derive(Clone)]
pub struct CachedTextMesh {
    pub mesh: TextMesh,
    pub slot_generation: u64,
    pub font_size: f32,
    pub wrap_width: Option<f32>,
    pub monospace_width: Option<f32>,
}

#[derive(Default)]
pub struct RetainedUiTextCache {
    pub mesh: Vec<Option<CachedTextMesh>>,
    pub mesh_dpi: f32,
    pub character_colors: HashMap<usize, Vec<Option<Vec4>>>,
    pub character_background_colors: HashMap<usize, Vec<Option<Vec4>>>,
}
