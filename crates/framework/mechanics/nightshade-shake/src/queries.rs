use crate::components::Trauma;

pub fn weight(trauma: &Trauma) -> f32 {
    trauma.0 * trauma.0
}

pub fn shaking(trauma: &Trauma) -> bool {
    trauma.0 > 0.0
}
