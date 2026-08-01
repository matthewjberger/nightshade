pub mod render_data;

use nalgebra_glm::Vec2;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, enum2schema::Schema)]
#[serde(default)]
pub struct Decal {
    pub texture: Option<String>,
    pub emissive_texture: Option<String>,
    pub emissive_strength: f32,
    pub color: [f32; 4],
    pub size: Vec2,
    pub depth: f32,
    pub normal_threshold: f32,
    pub fade_start: f32,
    pub fade_end: f32,
}

impl Default for Decal {
    fn default() -> Self {
        Self {
            texture: None,
            emissive_texture: None,
            emissive_strength: 1.0,
            color: [1.0, 1.0, 1.0, 1.0],
            size: Vec2::new(1.0, 1.0),
            depth: 1.0,
            normal_threshold: 0.5,
            fade_start: 50.0,
            fade_end: 100.0,
        }
    }
}

impl Decal {
    pub fn new(texture: impl Into<String>) -> Self {
        Self {
            texture: Some(texture.into()),
            ..Default::default()
        }
    }

    pub fn with_size(mut self, width: f32, height: f32) -> Self {
        self.size = Vec2::new(width, height);
        self
    }

    pub fn with_color(mut self, color: [f32; 4]) -> Self {
        self.color = color;
        self
    }

    pub fn with_emissive(mut self, texture: impl Into<String>, strength: f32) -> Self {
        self.emissive_texture = Some(texture.into());
        self.emissive_strength = strength;
        self
    }

    pub fn with_emissive_strength(mut self, strength: f32) -> Self {
        self.emissive_strength = strength;
        self
    }

    pub fn with_depth(mut self, depth: f32) -> Self {
        self.depth = depth;
        self
    }

    pub fn with_normal_threshold(mut self, threshold: f32) -> Self {
        self.normal_threshold = threshold;
        self
    }

    pub fn with_fade(mut self, start: f32, end: f32) -> Self {
        self.fade_start = start;
        self.fade_end = end;
        self
    }
}

#[derive(Default)]
pub struct DecalFrame {
    pub decals: Vec<render_data::DecalInstance>,
}
