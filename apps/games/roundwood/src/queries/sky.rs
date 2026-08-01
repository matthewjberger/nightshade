use crate::data;
use nalgebra_glm::Vec3;

pub fn tint(hour: f32) -> [f32; 3] {
    let hour = hour.rem_euclid(24.0);
    let mut previous = data::DAY_TINTS[0];
    for band in data::DAY_TINTS {
        if hour <= band.0 {
            let span = (band.0 - previous.0).max(f32::EPSILON);
            let share = ((hour - previous.0) / span).clamp(0.0, 1.0);
            return [
                previous.1[0] + (band.1[0] - previous.1[0]) * share,
                previous.1[1] + (band.1[1] - previous.1[1]) * share,
                previous.1[2] + (band.1[2] - previous.1[2]) * share,
            ];
        }
        previous = band;
    }
    previous.1
}

pub fn sun_direction(hour: f32) -> Vec3 {
    let share = ((hour - data::DAY_START) / (data::DAY_END - data::DAY_START)).clamp(0.0, 1.0);
    let angle = share * std::f32::consts::PI;
    nalgebra_glm::normalize(&Vec3::new(angle.cos(), angle.sin().max(0.12), 0.34))
}

pub fn sun_color(elevation: f32) -> Vec3 {
    let height = (elevation / 0.4).clamp(0.0, 1.0);
    let share = height * height * (3.0 - 2.0 * height);
    let warm = Vec3::new(data::SUN_WARM[0], data::SUN_WARM[1], data::SUN_WARM[2]);
    let white = Vec3::new(data::SUN_WHITE[0], data::SUN_WHITE[1], data::SUN_WHITE[2]);
    nalgebra_glm::lerp(&warm, &white, share)
}

pub fn clock(hour: f32) -> String {
    let hour = hour.rem_euclid(24.0);
    let whole = hour.floor() as u32;
    let minutes = ((hour - whole as f32) * 60.0).floor() as u32;
    let shown = match whole % 12 {
        0 => 12,
        other => other,
    };
    let suffix = if whole < 12 { "am" } else { "pm" };
    format!("{shown}:{minutes:02} {suffix}")
}
