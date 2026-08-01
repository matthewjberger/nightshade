use crate::data::PhaseLook;
use crate::resources::{Phase, PhaseSettings};

pub fn at(phase: &Phase) -> usize {
    phase.at
}

pub fn turning(phase: &Phase) -> bool {
    phase.applied != Some(phase.at)
}

pub fn look(settings: &PhaseSettings, at: usize) -> Option<PhaseLook> {
    settings.looks.get(at).copied()
}

pub fn faces(settings: &PhaseSettings) -> usize {
    settings.looks.len()
}
