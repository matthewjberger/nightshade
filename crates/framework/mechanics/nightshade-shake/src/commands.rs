use crate::components::Trauma;
use crate::resources::ShakeSettings;

pub fn shake(trauma: &mut Trauma, settings: &ShakeSettings, amount: f32) {
    trauma.0 = (trauma.0 + amount).clamp(0.0, settings.ceiling);
}

pub fn settle(trauma: &mut Trauma) {
    trauma.0 = 0.0;
}
