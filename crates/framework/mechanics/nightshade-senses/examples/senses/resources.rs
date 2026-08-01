use nightshade::prelude::Entity;

#[derive(Default)]
pub struct Field {
    pub hider: Option<Entity>,
    pub watchers: Vec<Entity>,
    pub crouching: bool,
    pub footfall: f32,
    pub shouts: u32,
}
