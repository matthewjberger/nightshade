use crate::data::Wash;
use nalgebra_glm::Vec4;
use nightshade::ecs::Entity;

#[derive(Default)]
pub struct Flash {
    pub washes: Vec<Wash>,
}

#[derive(Default, Clone, Copy, Debug)]
pub struct FlashHandles {
    pub panel: Entity,
}

#[derive(Default)]
pub struct FlashShown {
    pub color: Option<Vec4>,
}
