use crate::data::Sky;
use crate::resources::Weather;

pub fn raining(weather: &Weather) -> bool {
    matches!(weather.sky, Sky::Rain | Sky::Storm)
}

pub fn cover(weather: &Weather) -> f32 {
    match weather.sky {
        Sky::Clear => 0.0,
        Sky::Cloudy => 0.45,
        Sky::Rain => 0.8,
        Sky::Storm => 1.0,
    }
}
