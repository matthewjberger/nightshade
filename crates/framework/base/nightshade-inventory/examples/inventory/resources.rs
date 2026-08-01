use nightshade::prelude::Entity;

#[derive(Default)]
pub struct Bag {
    pub holder: Option<Entity>,
    pub posts: Vec<Entity>,
}
