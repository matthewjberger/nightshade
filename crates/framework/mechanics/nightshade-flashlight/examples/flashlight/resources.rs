use nightshade::prelude::Entity;

#[derive(Default)]
pub struct Torch {
    pub entity: Option<Entity>,
    pub sweep: f32,
    pub browned_out: u32,
    pub deaths: u32,
    pub said: String,
}
