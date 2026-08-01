use crate::resources::{WavePlan, WaveProgress};

pub fn current(plan: &WavePlan, progress: &WaveProgress) -> Option<crate::data::Wave> {
    plan.0.get(progress.wave).copied()
}

pub fn finished(plan: &WavePlan, progress: &WaveProgress) -> bool {
    progress.wave >= plan.0.len()
}
