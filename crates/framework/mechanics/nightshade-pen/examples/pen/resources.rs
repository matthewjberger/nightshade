use nightshade::prelude::Entity;

#[derive(Default, Clone, Copy)]
pub struct Yard {
    pub walker: Option<Entity>,
    pub free: Option<Entity>,
}
