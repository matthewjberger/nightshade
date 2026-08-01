use crate::data::Noise;
use nightshade_plane::Plane;

#[derive(Default)]
pub struct Noises {
    pub list: Vec<Noise>,
}

pub struct SensesSettings {
    pub plane: Plane,
    pub crouch_scale: f32,
    pub moving_scale: f32,
    pub half_angle: f32,
    pub noise_gain: f32,
    pub calm: f32,
    pub alarmed_above: f32,
}

impl Default for SensesSettings {
    fn default() -> Self {
        Self {
            plane: Plane::Ground,
            crouch_scale: 0.45,
            moving_scale: 1.4,
            half_angle: std::f32::consts::FRAC_PI_3,
            noise_gain: 0.6,
            calm: 0.35,
            alarmed_above: 0.5,
        }
    }
}
