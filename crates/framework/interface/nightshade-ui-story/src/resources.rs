use nalgebra_glm::{Vec4, vec4};
use nightshade::prelude::Entity;
use nightshade_story::Deed;

#[derive(Default)]
pub struct StoryCard {
    pub panel: Option<Entity>,
    pub lines: Vec<Entity>,
    pub links: Vec<Entity>,
    pub input: Option<Entity>,
    pub shown: Vec<String>,
    pub labels: Vec<String>,
    pub lit: Vec<bool>,
    pub asking: bool,
    pub offered: Vec<Deed>,
}

pub struct StoryStyle {
    pub width: f32,
    pub row_height: f32,
    pub padding: f32,
    pub spacing: f32,
    pub corner_radius: f32,
    pub border_width: f32,
    pub lines: usize,
    pub columns: usize,
    pub links: usize,
    pub line_size: f32,
    pub background: Vec4,
    pub border: Vec4,
    pub line_tint: Vec4,
}

impl Default for StoryStyle {
    fn default() -> Self {
        Self {
            width: 900.0,
            row_height: 28.0,
            padding: 26.0,
            spacing: 2.0,
            corner_radius: 12.0,
            border_width: 1.0,
            lines: 15,
            columns: 76,
            links: 5,
            line_size: 18.0,
            background: vec4(0.05, 0.05, 0.07, 0.94),
            border: vec4(0.46, 0.42, 0.62, 0.85),
            line_tint: vec4(0.90, 0.89, 0.84, 1.0),
        }
    }
}
