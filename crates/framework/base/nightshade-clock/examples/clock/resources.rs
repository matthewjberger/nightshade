use nightshade::prelude::Entity;

#[derive(Default)]
pub struct Dial {
    pub hand: Option<Entity>,
    pub marks: Vec<Entity>,
}
