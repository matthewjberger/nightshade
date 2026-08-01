use nalgebra_glm::IVec2;
use nightshade::prelude::Entity;

#[derive(Default)]
pub struct Walk {
    pub walker: Option<Entity>,
    pub marker: Option<Entity>,
    pub goal: IVec2,
    pub heading: IVec2,
    pub plant: bool,
}
