use crate::resources::DaySettings;
use nalgebra_glm::{Vec3, vec3};
use nightshade_clock::Clock;

pub fn night_share(clock: &Clock, settings: &DaySettings) -> f32 {
    let hour = clock.0.rem_euclid(24.0);
    if hour >= settings.dawn && hour <= settings.dusk {
        return 0.0;
    }
    let toward_dawn = (settings.dawn - hour).rem_euclid(24.0);
    let toward_dusk = (hour - settings.dusk).rem_euclid(24.0);
    let edge = toward_dawn.min(toward_dusk);
    (edge / 2.0).clamp(0.0, 1.0)
}

pub fn sun_direction(clock: &Clock) -> Vec3 {
    let turn = (clock.0.rem_euclid(24.0) / 24.0) * std::f32::consts::TAU;
    vec3(turn.cos(), -turn.sin().abs().max(0.1), turn.sin() * 0.3)
}

pub fn is_night(clock: &Clock, settings: &DaySettings) -> bool {
    night_share(clock, settings) > 0.5
}
