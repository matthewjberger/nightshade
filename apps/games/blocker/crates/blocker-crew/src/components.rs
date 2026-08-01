use crate::data::marks::MarkState;
use nalgebra_glm::Vec2;
use nightshade::ecs::Entity;

#[derive(Default, Clone, Copy, Debug)]
pub struct Offer(pub usize);

#[derive(Default, Clone, Copy, Debug)]
pub struct Shop(pub usize);

#[derive(Default, Clone, Copy, Debug)]
pub struct Persona(pub usize);

#[derive(Default, Clone, Copy, Debug)]
pub struct Haunt(pub Vec2);

#[derive(Default, Clone, Copy, Debug)]
pub struct Target(pub Vec2);

#[derive(Default, Clone, Copy, Debug)]
pub struct Post(pub Vec2);

#[derive(Clone, Copy, Debug)]
pub struct Speed(pub f32);

impl Default for Speed {
    fn default() -> Self {
        Self(2.0)
    }
}

#[derive(Default, Clone, Copy, Debug)]
pub struct Rest(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Mark(pub Entity);

#[derive(Default, Clone, Copy, Debug)]
pub struct Marked(pub MarkState);
