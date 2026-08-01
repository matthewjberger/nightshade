#[derive(Default)]
pub struct Board {
    pub standing: Vec<nightshade::prelude::Entity>,
    pub linked: usize,
    pub cut: usize,
    pub shaped: f32,
}
