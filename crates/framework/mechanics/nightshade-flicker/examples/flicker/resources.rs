use nightshade::prelude::Entity;

#[derive(Default)]
pub struct Street {
    pub lamps: Vec<Entity>,
    pub doused: u32,
}
