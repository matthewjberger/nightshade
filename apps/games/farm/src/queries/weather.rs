use crate::data::WeatherKind;
use crate::queries::clock::clock_text;
use crate::resources::Weather;
use crate::theme;

pub fn label(weather: &Weather) -> &'static str {
    if weather.active {
        weather.kind.name()
    } else {
        WeatherKind::Clear.name()
    }
}

pub fn announcement(kind: WeatherKind, active: bool) -> Option<&'static str> {
    match (kind, active) {
        (WeatherKind::Clear, _) => None,
        (WeatherKind::Cloudy, true) => Some("Clouds roll in"),
        (WeatherKind::Fog, true) => Some("Fog on the fields"),
        (WeatherKind::Fog, false) => Some("The fog lifts"),
        (WeatherKind::Wind, true) => Some("The wind picks up"),
        (WeatherKind::Rain, true) => Some("Rain sets in"),
        (WeatherKind::Rain, false) => Some("The rain clears"),
        (WeatherKind::Storm, true) => Some("A storm breaks"),
        (WeatherKind::Storm, false) => Some("The storm passes"),
        (_, false) => None,
    }
}

pub fn forecast(weather: &Weather) -> String {
    match (weather.kind, weather.window) {
        (WeatherKind::Clear, _) => "Clear skies today.".to_string(),
        (WeatherKind::Cloudy, _) => "Grey all day, but dry.".to_string(),
        (WeatherKind::Wind, _) => "Wind all day. Hold onto your hat.".to_string(),
        (WeatherKind::Fog, Some((_, end))) => format!("Fog until {}.", clock_text(end)),
        (WeatherKind::Rain, Some((start, end))) => {
            format!("Rain from {} until {}.", clock_text(start), clock_text(end))
        }
        (WeatherKind::Storm, Some((start, _))) => format!("A storm around {}.", clock_text(start)),
        (kind, None) => format!("{} today.", kind.name()),
    }
}

pub fn plant_lean(phase: f32, bend: f32, wind: f32, gust: f32) -> f32 {
    let breeze = (gust * 1.1 + phase).sin() * 0.3;
    let blow = (gust * 3.7 + phase).sin() * 0.5 + (gust * 6.1 + phase * 0.7).sin() * 0.5;
    theme::SWAY * bend * (breeze * 0.4 + blow * (0.3 + 2.0 * wind))
}
