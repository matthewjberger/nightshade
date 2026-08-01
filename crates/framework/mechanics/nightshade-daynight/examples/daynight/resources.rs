use nightshade::prelude::Entity;

#[derive(Default)]
pub struct Sky {
    pub sun: Option<Entity>,
    pub held: f32,
}
