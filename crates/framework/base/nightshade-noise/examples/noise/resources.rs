use nightshade::prelude::Entity;

#[derive(Default)]
pub struct Field {
    pub walker: Option<Entity>,
    pub columns: Vec<Entity>,
    pub stale: bool,
}
