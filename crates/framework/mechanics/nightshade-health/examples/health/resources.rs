use nightshade::prelude::Entity;

#[derive(Default)]
pub struct Rounds {
    pub bodies: Vec<Entity>,
    pub chosen: usize,
    pub fallen: u32,
    pub bounty: u32,
}
