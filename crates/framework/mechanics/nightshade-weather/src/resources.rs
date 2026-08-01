use crate::data::Sky;

#[derive(Default)]
pub struct Weather {
    pub sky: Sky,
    pub wetness: f32,
    pub remaining: f32,
}

pub struct WeatherSettings {
    pub shortest: f32,
    pub longest: f32,
    pub chance_of_rain: f32,
    pub dry_rate: f32,
}

impl Default for WeatherSettings {
    fn default() -> Self {
        Self {
            shortest: 30.0,
            longest: 180.0,
            chance_of_rain: 0.35,
            dry_rate: 0.08,
        }
    }
}
