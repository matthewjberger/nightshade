use crate::components::{Alert, Exposure};
use crate::data::Noise;
use crate::resources::{Noises, SensesSettings};
use nalgebra_glm::Vec3;

pub fn make_a_noise(noises: &mut Noises, at: Vec3, loudness: f32) {
    noises.list.push(Noise { at, loudness });
}

pub fn forget(noises: &mut Noises) {
    noises.list.clear();
}

pub fn hide(exposure: &mut Exposure, settings: &SensesSettings, crouching: bool, moving: bool) {
    let mut shown = 1.0;
    if crouching {
        shown *= settings.crouch_scale;
    }
    if moving {
        shown *= settings.moving_scale;
    }
    exposure.0 = shown.max(0.0);
}

pub fn alarm(alert: &mut Alert, amount: f32) {
    alert.0 = (alert.0 + amount).clamp(0.0, 1.0);
}

pub fn settle(alert: &mut Alert) {
    alert.0 = 0.0;
}
