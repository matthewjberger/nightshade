use crate::components::{FlickerBase, Flickering};
use crate::resources::FlickerSettings;

pub fn level(flickering: &Flickering) -> f32 {
    flickering.0
}

pub fn intensity(base: &FlickerBase, flickering: &Flickering) -> f32 {
    base.0 * flickering.0
}

pub fn dark(flickering: &Flickering, settings: &FlickerSettings) -> bool {
    flickering.0 < settings.dark_below
}
