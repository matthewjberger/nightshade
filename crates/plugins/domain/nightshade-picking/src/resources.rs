use crate::contract::{PickingRay, PickingResult};

#[derive(Default)]
pub struct Picked {
    pub under: Option<PickingResult>,
    pub ray: Option<PickingRay>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum PickWhen {
    #[default]
    PointerMoved,
    EveryFrame,
    Never,
}
