use std::collections::{HashMap, HashSet};

use kira::{
    AudioManager, AudioManagerSettings, Capacities, DefaultBackend,
    listener::ListenerHandle,
    modulator::tweener::TweenerHandle,
    sound::static_sound::{StaticSoundData, StaticSoundHandle},
    track::{SendTrackHandle, SpatialTrackHandle, TrackHandle},
};
use nightshade_ecs::Entity;

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
    pub sound_handles: HashMap<Entity, StaticSoundHandle>,
    pub spatial_tracks: HashMap<Entity, SpatialTrackHandle>,
    pub spatial_sound_handles: HashMap<Entity, StaticSoundHandle>,
    pub paused_handles: HashSet<Entity>,
}

pub fn audio_engine_initialize(engine: &mut AudioEngine) -> Result<(), Box<dyn std::error::Error>> {
    if engine.initialized {
        return Ok(());
    }

    let settings = AudioManagerSettings::<DefaultBackend> {
        capacities: Capacities {
            sub_track_capacity: 256,
            send_track_capacity: 64,
            clock_capacity: 16,
            modulator_capacity: 64,
            listener_capacity: 8,
        },
        ..Default::default()
    };

    match AudioManager::<DefaultBackend>::new(settings) {
        Ok(manager) => {
            engine.manager = Some(manager);
            engine.initialized = true;
            tracing::info!("Audio engine initialized");
            Ok(())
        }
        Err(error) => {
            tracing::error!("Failed to initialize audio engine: {}", error);
            Err(Box::new(error))
        }
    }
}

pub fn audio_engine_is_initialized(engine: &AudioEngine) -> bool {
    engine.initialized && engine.manager.is_some()
}

pub fn audio_engine_load_sound(
    engine: &mut AudioEngine,
    name: impl Into<String>,
    data: StaticSoundData,
) {
    engine.sound_cache.insert(name.into(), data);
}

pub fn audio_engine_get_sound<'a>(
    engine: &'a AudioEngine,
    name: &str,
) -> Option<&'a StaticSoundData> {
    engine.sound_cache.get(name)
}

pub fn audio_engine_has_handle(engine: &AudioEngine, entity: Entity) -> bool {
    engine.sound_handles.contains_key(&entity) || engine.spatial_tracks.contains_key(&entity)
}

pub fn audio_engine_stop_sound(engine: &mut AudioEngine, entity: Entity) {
    use kira::Tween;
    use std::time::Duration;
    let fade = Tween {
        duration: Duration::from_millis(50),
        ..Default::default()
    };
    if let Some(handle) = engine.sound_handles.get_mut(&entity) {
        handle.stop(fade);
    }
    engine.sound_handles.remove(&entity);
    engine.spatial_tracks.remove(&entity);
    engine.spatial_sound_handles.remove(&entity);
    engine.paused_handles.remove(&entity);
}

pub fn audio_engine_pause_sound(engine: &mut AudioEngine, entity: Entity) {
    use kira::Tween;
    use std::time::Duration;
    let fade = Tween {
        duration: Duration::from_millis(20),
        ..Default::default()
    };
    if let Some(handle) = engine.sound_handles.get_mut(&entity) {
        handle.pause(fade);
    }
    if let Some(handle) = engine.spatial_sound_handles.get_mut(&entity) {
        handle.pause(fade);
    }
}

pub fn audio_engine_resume_sound(engine: &mut AudioEngine, entity: Entity) {
    use kira::Tween;
    use std::time::Duration;
    let fade = Tween {
        duration: Duration::from_millis(20),
        ..Default::default()
    };
    if let Some(handle) = engine.sound_handles.get_mut(&entity) {
        handle.resume(fade);
    }
    if let Some(handle) = engine.spatial_sound_handles.get_mut(&entity) {
        handle.resume(fade);
    }
}

pub fn audio_engine_fade_volume(
    engine: &mut AudioEngine,
    entity: Entity,
    volume: f32,
    seconds: f32,
) {
    use kira::{Decibels, Tween};
    use std::time::Duration;
    let tween = Tween {
        duration: Duration::from_secs_f32(seconds.max(0.0)),
        ..Default::default()
    };
    let target = Decibels::from(volume.max(0.0));
    if let Some(handle) = engine.sound_handles.get_mut(&entity) {
        handle.set_volume(target, tween);
    }
    if let Some(handle) = engine.spatial_sound_handles.get_mut(&entity) {
        handle.set_volume(target, tween);
    }
}

pub fn audio_engine_cleanup_entity(engine: &mut AudioEngine, entity: Entity) {
    audio_engine_stop_sound(engine, entity);
}
