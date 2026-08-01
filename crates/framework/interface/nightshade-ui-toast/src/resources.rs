use nalgebra_glm::{Vec4, vec4};
use nightshade::prelude::Entity;

#[derive(Default)]
pub struct ToastCard {
    pub card: Option<Entity>,
    pub line: Option<Entity>,
    pub shown: String,
}

pub struct ToastStyle {
    pub width: f32,
    pub height: f32,
    pub down: f32,
    pub text_size: f32,
    pub background: Vec4,
    pub border: Vec4,
    pub text: Vec4,
}

impl Default for ToastStyle {
    fn default() -> Self {
        Self {
            width: 460.0,
            height: 52.0,
            down: 96.0,
            text_size: 20.0,
            background: vec4(0.05, 0.05, 0.08, 0.86),
            border: vec4(0.85, 0.72, 0.45, 0.9),
            text: vec4(0.94, 0.92, 0.88, 1.0),
        }
    }
}
