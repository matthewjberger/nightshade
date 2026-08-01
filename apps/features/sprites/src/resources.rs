use nightshade::prelude::Entity;

#[derive(Default)]
pub struct Farm {
    pub camera: Option<Entity>,
    pub farmer: Option<Entity>,
}
