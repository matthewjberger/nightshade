use crate::components::{FlickerBase, Flickering};

pub fn steady(flickering: &mut Flickering) {
    flickering.0 = 1.0;
}

pub fn douse(flickering: &mut Flickering) {
    flickering.0 = 0.0;
}

pub fn set_base(base: &mut FlickerBase, intensity: f32) {
    base.0 = intensity.max(0.0);
}
