use crate::resources::{Phase, PhaseSettings};

pub fn turn_to(phase: &mut Phase, at: usize) {
    phase.at = at;
}

pub fn turn_over(phase: &mut Phase, settings: &PhaseSettings) {
    if settings.looks.is_empty() {
        return;
    }
    phase.at = (phase.at + 1) % settings.looks.len();
}

pub fn apply_again(phase: &mut Phase) {
    phase.applied = None;
}
