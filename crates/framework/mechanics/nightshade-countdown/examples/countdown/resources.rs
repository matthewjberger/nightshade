use nightshade::prelude::Entity;

#[derive(Default)]
pub struct Rounds {
    pub bar: Option<Entity>,
    pub rounds: u32,
}
