use crate::commands::roll_the_sky;
use crate::queries::raining;
use crate::resources::{Weather, WeatherSettings};
use nightshade::prelude::*;
use nightshade_platform::contract::time::Time;
use nightshade_random::Random;

pub fn turn_the_weather(
    time: Res<Time>,
    settings: Res<WeatherSettings>,
    mut random: ResMut<Random>,
    mut weather: ResMut<Weather>,
) {
    let delta = time.delta_time;
    weather.remaining -= delta;
    if weather.remaining <= 0.0 {
        roll_the_sky(&mut weather, &settings, &mut random);
    }
    let wetter = raining(&weather);
    weather.wetness = if wetter {
        (weather.wetness + settings.dry_rate * delta).min(1.0)
    } else {
        (weather.wetness - settings.dry_rate * delta).max(0.0)
    };
}
