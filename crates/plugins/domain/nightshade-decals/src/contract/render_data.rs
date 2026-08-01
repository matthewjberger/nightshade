use nalgebra_glm::{Mat4, Vec2};

use crate::contract::Decal;

#[derive(Clone, Debug)]
pub struct DecalInstance {
    pub texture: Option<String>,
    pub emissive_texture: Option<String>,
    pub emissive_strength: f32,
    pub color: [f32; 4],
    pub size: Vec2,
    pub depth: f32,
    pub normal_threshold: f32,
    pub fade_start: f32,
    pub fade_end: f32,
    pub transform: Mat4,
    pub visible: bool,
}

impl DecalInstance {
    pub fn new(decal: &Decal, transform: Mat4, visible: bool) -> Self {
        Self {
            texture: decal.texture.clone(),
            emissive_texture: decal.emissive_texture.clone(),
            emissive_strength: decal.emissive_strength,
            color: decal.color,
            size: decal.size,
            depth: decal.depth,
            normal_threshold: decal.normal_threshold,
            fade_start: decal.fade_start,
            fade_end: decal.fade_end,
            transform,
            visible,
        }
    }
}
