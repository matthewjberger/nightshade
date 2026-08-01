use crate::contract::data::text_data::{FontKind, TextAlignment, VerticalAlignment};
use nalgebra_glm::{Vec2, Vec4};

use crate::contract::data::text_data::TextMesh;

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct TextCharacterColors {
    pub colors: Vec<Option<Vec4>>,
    pub dirty: bool,
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct TextCharacterBackgroundColors {
    pub colors: Vec<Option<Vec4>>,
    pub dirty: bool,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Text {
    pub text_index: usize,
    pub properties: TextProperties,
    pub dirty: bool,
    #[serde(skip)]
    pub cached_mesh: Option<TextMesh>,
    pub billboard: bool,
}

impl Default for Text {
    fn default() -> Self {
        Self {
            text_index: 0,
            properties: TextProperties::default(),
            dirty: true,
            cached_mesh: None,
            billboard: false,
        }
    }
}

impl Text {
    pub fn new(text_index: usize) -> Self {
        Self {
            text_index,
            properties: TextProperties::default(),
            dirty: true,
            cached_mesh: None,
            billboard: false,
        }
    }

    pub fn with_properties(mut self, properties: TextProperties) -> Self {
        self.properties = properties;
        self.dirty = true;
        self
    }

    pub fn get_bounds(&self) -> Option<Vec2> {
        self.cached_mesh.as_ref().map(|mesh| mesh.bounds)
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, enum2schema::Schema)]
pub struct TextProperties {
    pub font_size: f32,
    pub color: nalgebra_glm::Vec4,
    pub alignment: TextAlignment,
    pub vertical_alignment: VerticalAlignment,
    pub line_height: f32,
    pub letter_spacing: f32,
    pub outline_width: f32,
    pub outline_color: nalgebra_glm::Vec4,
    pub smoothing: f32,
    pub monospace_width: Option<f32>,
    pub anchor_character: Option<usize>,
    pub font_kind: FontKind,
}

impl Default for TextProperties {
    fn default() -> Self {
        Self {
            font_size: 16.0,
            color: nalgebra_glm::Vec4::new(1.0, 1.0, 1.0, 1.0),
            alignment: TextAlignment::Left,
            vertical_alignment: VerticalAlignment::Baseline,
            line_height: 1.2,
            letter_spacing: 0.0,
            outline_width: 0.0,
            outline_color: nalgebra_glm::Vec4::new(0.0, 0.0, 0.0, 1.0),
            smoothing: 0.0,
            monospace_width: None,
            anchor_character: None,
            font_kind: FontKind::Default,
        }
    }
}
