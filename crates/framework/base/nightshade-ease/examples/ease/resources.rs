use nightshade::prelude::Entity;

#[derive(Default)]
pub struct Run {
    pub runners: Vec<Entity>,
    pub turn: f32,
}
