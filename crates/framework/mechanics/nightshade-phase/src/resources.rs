use crate::data::PhaseLook;

#[derive(Default)]
pub struct Phase {
    pub at: usize,
    pub applied: Option<usize>,
}

#[derive(Default)]
pub struct PhaseSettings {
    pub looks: Vec<PhaseLook>,
}
