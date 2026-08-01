use crate::data::twilight::Twilight;

pub fn night_factor(hour: f32, twilight: &Twilight) -> f32 {
    if hour < twilight.dawn_start || hour > twilight.dusk_end {
        1.0
    } else if hour < twilight.dawn_end {
        let span = (twilight.dawn_end - twilight.dawn_start).max(f32::EPSILON);
        smoothstep(1.0 - (hour - twilight.dawn_start) / span)
    } else if hour > twilight.dusk_start {
        let span = (twilight.dusk_end - twilight.dusk_start).max(f32::EPSILON);
        smoothstep((hour - twilight.dusk_start) / span)
    } else {
        0.0
    }
}

pub fn golden_factor(hour: f32, twilight: &Twilight) -> f32 {
    let dawn = 1.0 - ((hour - twilight.dawn_end) / 1.5).abs();
    let dusk = 1.0 - ((hour - twilight.dusk_start) / 1.5).abs();
    dawn.max(dusk).clamp(0.0, 1.0)
}

pub fn smoothstep(value: f32) -> f32 {
    let clamped = value.clamp(0.0, 1.0);
    clamped * clamped * (3.0 - 2.0 * clamped)
}
