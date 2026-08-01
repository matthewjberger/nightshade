#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum WeatherKind {
    #[default]
    Clear,
    Cloudy,
    Fog,
    Drizzle,
    Rain,
    Storm,
    Snowfall,
    Blizzard,
}

#[derive(Clone, Copy)]
pub struct WeatherInfo {
    pub rain: f32,
    pub snow: f32,
    pub cloud: f32,
    pub fog: f32,
    pub wind: f32,
    pub word: &'static str,
}

const fn weather(
    rain: f32,
    snow: f32,
    cloud: f32,
    fog: f32,
    wind: f32,
    word: &'static str,
) -> WeatherInfo {
    WeatherInfo {
        rain,
        snow,
        cloud,
        fog,
        wind,
        word,
    }
}

pub const WEATHERS: [WeatherInfo; 8] = [
    weather(0.0, 0.0, 0.0, 0.0, 0.05, "The sky clears"),
    weather(0.0, 0.0, 0.55, 0.1, 0.15, "Clouds roll in"),
    weather(0.0, 0.0, 0.5, 1.0, 0.0, "A fog settles over the land"),
    weather(0.4, 0.0, 0.7, 0.35, 0.1, "A light drizzle begins"),
    weather(1.0, 0.0, 0.85, 0.2, 0.2, "Rain starts to fall"),
    weather(1.0, 0.0, 1.0, 0.25, 0.55, "A storm breaks"),
    weather(0.0, 0.6, 0.75, 0.35, 0.15, "Snow starts to fall"),
    weather(0.0, 1.0, 1.0, 0.7, 0.8, "A blizzard howls in"),
];

pub const SEASONS: [&str; 4] = ["Spring", "Summer", "Autumn", "Winter"];

impl WeatherKind {
    pub fn rain(self) -> f32 {
        WEATHERS[self as usize].rain
    }

    pub fn snow(self) -> f32 {
        WEATHERS[self as usize].snow
    }

    pub fn clouds(self) -> f32 {
        WEATHERS[self as usize].cloud
    }

    pub fn fog(self) -> f32 {
        WEATHERS[self as usize].fog
    }

    pub fn wind(self) -> f32 {
        WEATHERS[self as usize].wind
    }

    pub fn word(self) -> &'static str {
        WEATHERS[self as usize].word
    }
}

pub const DAWN_HOUR: f32 = 5.5;

pub const DUSK_HOUR: f32 = 18.5;
