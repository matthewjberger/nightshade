use nalgebra_glm::IVec2;
use nightshade_ecs::Storage;

pub type WalkableAnswer = fn(&Storage, IVec2) -> bool;
pub type StepCostAnswer = fn(&Storage, IVec2) -> f32;
pub type InBoundsAnswer = fn(&Storage, IVec2) -> bool;

#[derive(Default)]
pub struct Cells {
    pub walkable: Option<WalkableAnswer>,
    pub step_cost: Option<StepCostAnswer>,
    pub in_bounds: Option<InBoundsAnswer>,
}

pub struct CellSize(pub f32);

impl Default for CellSize {
    fn default() -> Self {
        Self(1.0)
    }
}
