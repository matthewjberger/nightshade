use nightshade::prelude::Entity;

#[derive(Default)]
pub struct Hammer {
    pub entity: Option<Entity>,
    pub over: usize,
    pub struck: u32,
}
