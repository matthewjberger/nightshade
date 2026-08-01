use crate::components::{Bob, Phase};
use crate::resources::HoverSettings;

pub fn lift(bob: &Bob, phase: &Phase, settings: &HoverSettings, clock: f32) -> f32 {
    (clock * settings.rate + phase.0).sin() * bob.0
}
