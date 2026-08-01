use crate::data::Sky;
use crate::resources::{Weather, WeatherSettings};
use nightshade_random::Random;
use nightshade_random::commands::{between, chance};

pub fn roll_the_sky(weather: &mut Weather, settings: &WeatherSettings, random: &mut Random) {
    weather.sky = if chance(random, settings.chance_of_rain) {
        if chance(random, 0.3) {
            Sky::Storm
        } else {
            Sky::Rain
        }
    } else if chance(random, 0.4) {
        Sky::Cloudy
    } else {
        Sky::Clear
    };
    weather.remaining = between(random, settings.shortest, settings.longest);
}
