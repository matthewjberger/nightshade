use nalgebra_glm::{Vec4, vec4};
use nightshade::prelude::Entity;

#[derive(Default)]
pub struct DialogueCard {
    pub card: Option<Entity>,
    pub speaker: Option<Entity>,
    pub line: Option<Entity>,
    pub shown: String,
    pub named: String,
}

pub struct DialogueStyle {
    pub width: f32,
    pub height: f32,
    pub up: f32,
    pub name_size: f32,
    pub line_size: f32,
    pub background: Vec4,
    pub border: Vec4,
    pub name_tint: Vec4,
    pub line_tint: Vec4,
}

impl Default for DialogueStyle {
    fn default() -> Self {
        Self {
            width: 720.0,
            height: 140.0,
            up: 72.0,
            name_size: 20.0,
            line_size: 22.0,
            background: vec4(0.06, 0.06, 0.09, 0.92),
            border: vec4(0.62, 0.58, 0.82, 0.9),
            name_tint: vec4(0.78, 0.72, 0.96, 1.0),
            line_tint: vec4(0.94, 0.93, 0.90, 1.0),
        }
    }
}
