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
    /// When true while `playing` is also true, the running sound is held at
    /// its current position. Clearing it resumes from where it stopped, unlike
    /// `playing = false`, which frees the sound and restarts from the beginning.
    #[serde(default)]
    pub paused: bool,
    pub spatial: bool,
    pub bus: AudioBus,
    pub min_distance: f32,
    pub max_distance: f32,
    pub reverb_zones: Vec<(String, f32)>,
    /// Optional list of additional clip paths. When non-empty and
    /// `random_pick` is true, the audio system selects one of
    /// `audio_ref` + `random_clips` at random each time playback
    /// transitions from stopped to playing. Useful for footstep packs,
    /// random stingers, or one-shot scares.
    #[serde(default)]
    pub random_clips: Vec<String>,
    /// When true, the source treats `audio_ref` plus `random_clips` as
    /// a pool and chooses one at random when starting playback.
    #[serde(default)]
    pub random_pick: bool,
    /// Playback rate factor applied when the sound starts. 1.0 plays at
    /// the original pitch and speed. 2.0 plays an octave higher and
    /// twice as fast. 0.5 plays an octave lower and at half speed.
    #[serde(default = "default_playback_rate")]
    pub playback_rate: f64,
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
            reverb_zones: Vec::new(),
            random_clips: Vec::new(),
            random_pick: false,
            playback_rate: 1.0,
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
