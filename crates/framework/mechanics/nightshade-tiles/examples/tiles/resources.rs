use nalgebra_glm::IVec2;
use nightshade::prelude::Entity;

#[derive(Default)]
pub struct Field {
    pub cells: Vec<(IVec2, Entity)>,
    pub cursor: Option<Entity>,
    pub at: IVec2,
    pub reached: Vec<IVec2>,
}
