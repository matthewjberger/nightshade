use std::collections::{HashMap, HashSet};

use kira::{
    AudioManager, DefaultBackend,
    effect::{filter::FilterHandle, volume_control::VolumeControlHandle},
    listener::ListenerHandle,
    modulator::tweener::TweenerHandle,
    sound::static_sound::{StaticSoundData, StaticSoundHandle},
    track::{SendTrackHandle, SpatialTrackHandle, TrackHandle},
};
use nightshade::ecs::Entity;

pub type Sounds = nightshade_load::Loader<crate::data::Sound>;

#[derive(Default)]
pub struct AudioBuses {
    pub master: Option<TrackHandle>,
    pub music: Option<TrackHandle>,
    pub sfx: Option<TrackHandle>,
    pub ambient: Option<TrackHandle>,
    pub voice: Option<TrackHandle>,
    pub ui: Option<TrackHandle>,
}

#[derive(Default)]
pub struct AudioEngine {
    pub manager: Option<AudioManager<DefaultBackend>>,
    pub listener: Option<ListenerHandle>,
    pub buses: AudioBuses,
    pub buses_built: bool,
    pub reverb_sends: HashMap<String, SendTrackHandle>,
    pub voice_ducking: Option<TweenerHandle>,
    pub initialized: bool,
    pub sound_cache: HashMap<String, StaticSoundData>,
    pub sound_sources: HashMap<String, Vec<u8>>,
    pub sound_handles: HashMap<Entity, StaticSoundHandle>,
    pub spatial_tracks: HashMap<Entity, SpatialTrackHandle>,
    pub spatial_sound_handles: HashMap<Entity, StaticSoundHandle>,
    pub spatial_filters: HashMap<Entity, FilterHandle>,
    #[cfg(not(target_arch = "wasm32"))]
    pub spatial_aims: HashMap<Entity, crate::data::binaural::BinauralHandle>,
    pub listener_place: nalgebra_glm::Vec3,
    pub listener_turn: nalgebra_glm::Quat,
    pub paused_handles: HashSet<Entity>,
    pub last_clips: HashMap<Entity, String>,
    #[cfg(not(target_arch = "wasm32"))]
    pub streams: HashMap<Entity, crate::data::stream::StreamHandle>,
    #[cfg(not(target_arch = "wasm32"))]
    pub captures: HashMap<Entity, crate::data::capture::Capture>,
    #[cfg(not(target_arch = "wasm32"))]
    pub room_reverb: Option<crate::data::room_reverb::RoomReverbHandle>,
    #[cfg(not(target_arch = "wasm32"))]
    pub zone_reverbs: Vec<crate::data::room_reverb::RoomReverbHandle>,
    pub room_level: Option<VolumeControlHandle>,
}
