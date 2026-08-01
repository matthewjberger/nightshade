#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AudioTrack {
    #[default]
    Sound,
    Music,
}

#[derive(Debug, Clone, Default)]
pub struct PendingSound {
    pub clip: String,
    pub volume: f32,
    pub track: AudioTrack,
}

use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize, enum2schema::Schema,
)]
#[schema(string_enum)]
pub enum AudioBus {
    Master,
    Music,
    #[default]
    Sfx,
    Ambient,
    Voice,
    Ui,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, enum2schema::Schema)]
pub struct AudioSource {
    pub audio_ref: Option<String>,
    pub volume: f32,
    pub looping: bool,
    pub playing: bool,
    #[serde(default)]
    pub paused: bool,
    pub spatial: bool,
    pub bus: AudioBus,
    pub min_distance: f32,
    pub max_distance: f32,
    pub rolloff: f32,
    pub reverb_zones: Vec<(String, f32)>,
    #[serde(default)]
    pub random_clips: Vec<String>,
    #[serde(default)]
    pub random_pick: bool,
    #[serde(default = "default_playback_rate")]
    pub playback_rate: f64,
    #[serde(default = "default_spread")]
    pub pitch_spread: f32,
    #[serde(default)]
    pub volume_spread: f32,
    #[serde(default)]
    pub no_repeat: bool,
    #[serde(default)]
    pub live: bool,
    #[serde(default = "default_rate")]
    pub live_rate: u32,
}

fn default_rate() -> u32 {
    44_100
}

fn default_spread() -> f32 {
    1.0
}

fn default_playback_rate() -> f64 {
    1.0
}

impl Default for AudioSource {
    fn default() -> Self {
        Self {
            audio_ref: None,
            volume: 1.0,
            looping: false,
            playing: false,
            paused: false,
            spatial: false,
            bus: AudioBus::Sfx,
            min_distance: 1.0,
            max_distance: 50.0,
            rolloff: 2.0,
            reverb_zones: Vec::new(),
            random_clips: Vec::new(),
            random_pick: false,
            playback_rate: 1.0,
            pitch_spread: 1.0,
            volume_spread: 0.0,
            no_repeat: false,
            live: false,
            live_rate: 44_100,
        }
    }
}

impl AudioSource {
    pub fn new(audio_ref: impl Into<String>) -> Self {
        Self {
            audio_ref: Some(audio_ref.into()),
            ..Default::default()
        }
    }

    pub fn with_volume(mut self, volume: f32) -> Self {
        self.volume = volume;
        self
    }

    pub fn with_looping(mut self, looping: bool) -> Self {
        self.looping = looping;
        self
    }

    pub fn with_spatial(mut self, spatial: bool) -> Self {
        self.spatial = spatial;
        self
    }

    pub fn with_bus(mut self, bus: AudioBus) -> Self {
        self.bus = bus;
        self
    }

    pub fn with_distance(mut self, min: f32, max: f32) -> Self {
        self.min_distance = min;
        self.max_distance = max;
        self
    }

    pub fn with_reverb(mut self, reverb: bool) -> Self {
        self.reverb_zones.retain(|(name, _)| name != "default");
        if reverb {
            self.reverb_zones.push(("default".to_string(), 0.0));
        }
        self
    }

    pub fn with_reverb_zone(mut self, zone: impl Into<String>, send_decibels: f32) -> Self {
        let zone = zone.into();
        self.reverb_zones.retain(|(name, _)| name != &zone);
        self.reverb_zones.push((zone, send_decibels));
        self
    }

    pub fn playing(mut self) -> Self {
        self.playing = true;
        self
    }

    pub fn with_paused(mut self, paused: bool) -> Self {
        self.paused = paused;
        self
    }

    pub fn with_playback_rate(mut self, rate: f64) -> Self {
        self.playback_rate = rate;
        self
    }
}

#[derive(Debug, Clone, Copy, Default, serde::Serialize, serde::Deserialize)]
pub struct AudioListener;

impl AudioSource {
    pub fn with_rolloff(mut self, rolloff: f32) -> Self {
        self.rolloff = rolloff;
        self
    }

    pub fn with_spread(mut self, pitch_spread: f32, volume_spread: f32) -> Self {
        self.pitch_spread = pitch_spread;
        self.volume_spread = volume_spread;
        self
    }

    pub fn without_repeats(mut self) -> Self {
        self.no_repeat = true;
        self
    }

    pub fn live(mut self, rate: u32) -> Self {
        self.live = true;
        self.live_rate = rate;
        self
    }
}
