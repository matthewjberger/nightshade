use nightshade::prelude::Entity;

#[derive(Default)]
pub struct Row {
    pub coins: Vec<Entity>,
    pub chosen: usize,
}
