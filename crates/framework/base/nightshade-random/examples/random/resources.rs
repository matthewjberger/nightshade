use nightshade::prelude::Entity;

#[derive(Default)]
pub struct Scatter {
    pub discs: Vec<Entity>,
    pub rings: Vec<Entity>,
    pub spread: f32,
    pub heads: usize,
    pub asked: bool,
}
