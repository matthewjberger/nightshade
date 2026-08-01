use crate::theme;
use nalgebra_glm::Vec3;

pub struct Sky {
    pub horizon: Vec3,
    pub ambient: Vec3,
    pub daylight: f32,
    pub glow: f32,
}

fn smooth(edge_low: f32, edge_high: f32, value: f32) -> f32 {
    let step = ((value - edge_low) / (edge_high - edge_low)).clamp(0.0, 1.0);
    step * step * (3.0 - 2.0 * step)
}

pub fn sun_elevation(hour: f32) -> f32 {
    ((hour - 6.0) / 12.0 * std::f32::consts::PI).sin()
}

pub fn sky(hour: f32, cloud: f32, fog: f32) -> Sky {
    let elevation = sun_elevation(hour);
    let daylight = smooth(-0.08, 0.3, elevation);
    let glow = (-(elevation / 0.14).powi(2)).exp() * smooth(-0.25, -0.02, elevation).max(0.35);
    let horizon = nalgebra_glm::lerp(&theme::SKY_NIGHT_HORIZON, &theme::SKY_DAY_HORIZON, daylight);
    let horizon = nalgebra_glm::lerp(&horizon, &theme::SKY_DUSK_HORIZON, glow);
    let grey = Vec3::new(0.56, 0.59, 0.64) * (0.15 + 0.85 * daylight);
    let horizon = nalgebra_glm::lerp(&horizon, &grey, cloud * 0.7);
    let pale = Vec3::new(0.82, 0.84, 0.88) * (0.1 + 0.9 * daylight);
    let horizon = nalgebra_glm::lerp(&horizon, &pale, fog);
    let lit = ((elevation - theme::AMBIENT_FLOOR_ELEVATION)
        / (1.0 - theme::AMBIENT_FLOOR_ELEVATION))
        .clamp(0.0, 1.0);
    let ambient = nalgebra_glm::lerp(&theme::AMBIENT_NIGHT, &theme::AMBIENT_DAY, lit)
        + theme::AMBIENT_DUSK * glow;
    let ambient = ambient * (1.0 - 0.28 * cloud);
    Sky {
        horizon,
        ambient,
        daylight,
        glow,
    }
}
