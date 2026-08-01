use crate::data::chat::{CHANNELS, Channel};
use blocker_weather::data::weather::WeatherKind;
use blocker_world::data::world::biomes::Biome;
use std::collections::VecDeque;

#[derive(Clone)]
pub struct ChatLine {
    pub channel: Channel,
    pub speaker: Option<String>,
    pub text: String,
}

pub struct Scheduled {
    pub delay: f32,
    pub line: ChatLine,
}

#[derive(Clone)]
pub struct Speaker {
    pub name: String,
    pub voice: usize,
    pub class: usize,
}

pub struct Fill {
    pub speakers: Vec<Speaker>,
    pub monster: String,
    pub place: String,
    pub item: String,
    pub price: String,
    pub quest: String,
    pub guild: String,
    pub player: String,
    pub level: String,
    pub killer: String,
}

#[derive(Default)]
pub struct Chat {
    pub lines: VecDeque<(Channel, String)>,
    pub counts: [u64; CHANNELS],
}

pub struct Chatter {
    pub scheduled: Vec<Scheduled>,
    pub gap: f32,
    pub chatter_timer: f32,
    pub whisper_timer: f32,
    pub greet_timer: f32,
}

impl Default for Chatter {
    fn default() -> Self {
        Self {
            scheduled: Vec::new(),
            gap: 0.0,
            chatter_timer: 4.0,
            whisper_timer: 30.0,
            greet_timer: 0.0,
        }
    }
}

#[derive(Default)]
pub struct Roster(pub Vec<Speaker>);

#[derive(Default, Clone, Copy)]
pub struct Noted {
    pub weather: Option<WeatherKind>,
    pub night: Option<bool>,
    pub biome: Option<Biome>,
}
