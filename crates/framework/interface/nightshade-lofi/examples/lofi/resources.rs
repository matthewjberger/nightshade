use nightshade::prelude::Entity;

#[derive(Default)]
pub struct Stage {
    pub ball: Option<Entity>,
    pub spin: f32,
    pub wearing: &'static str,
}
