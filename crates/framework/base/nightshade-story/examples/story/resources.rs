use nightshade::prelude::Entity;

#[derive(Default)]
pub struct Board {
    pub rooms: Vec<Entity>,
    pub things: Vec<Entity>,
}
