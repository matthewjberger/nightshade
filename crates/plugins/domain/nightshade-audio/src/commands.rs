use crate::resources::AudioEngine;
use kira::{
    AudioManager, AudioManagerSettings, Capacities, DefaultBackend, Tween,
    sound::static_sound::StaticSoundData,
};
use nightshade::ecs::Entity;
use std::io::Cursor;
use std::time::Duration;
use tracing;

#[cfg(not(target_arch = "wasm32"))]
use crate::contract::AudioBus;
#[cfg(not(target_arch = "wasm32"))]
use crate::queries::bus_handle_mut;
#[cfg(not(target_arch = "wasm32"))]
use kira::Decibels;

#[cfg(not(target_arch = "wasm32"))]
pub fn set_audio_bus_volume(
    engine: &mut AudioEngine,
    bus: AudioBus,
    decibels: f32,
    fade_seconds: f32,
) {
    if let Some(handle) = bus_handle_mut(engine, bus) {
        handle.set_volume(
            Decibels(decibels),
            Tween {
                duration: Duration::from_secs_f32(fade_seconds.max(0.0)),
                ..Default::default()
            },
        );
    }
}

pub fn set_voice_ducking(engine: &mut AudioEngine, ducking: f32, fade_seconds: f32) {
    if let Some(tweener) = engine.voice_ducking.as_mut() {
        tweener.set(
            ducking.clamp(0.0, 1.0) as f64,
            Tween {
                duration: Duration::from_secs_f32(fade_seconds.max(0.0)),
                ..Default::default()
            },
        );
    }
}

pub fn load_sound_from_bytes(
    bytes: &'static [u8],
) -> Result<StaticSoundData, Box<dyn std::error::Error>> {
    let cursor = Cursor::new(bytes);
    let sound_data = StaticSoundData::from_cursor(cursor)?;
    Ok(sound_data)
}

pub fn audio_engine_load_sound_from_bytes(
    engine: &mut AudioEngine,
    name: impl Into<String>,
    bytes: Vec<u8>,
) -> bool {
    let name = name.into();
    match load_sound_from_cursor(bytes.clone()) {
        Ok(data) => {
            engine.sound_cache.insert(name.clone(), data);
            engine.sound_sources.insert(name, bytes);
            true
        }
        Err(error) => {
            tracing::error!("Failed to load sound {name}: {error}");
            false
        }
    }
}

pub fn load_sound_from_cursor<T: AsRef<[u8]> + Send + Sync + 'static>(
    data: T,
) -> Result<StaticSoundData, Box<dyn std::error::Error>> {
    let cursor = Cursor::new(data);
    let sound_data = StaticSoundData::from_cursor(cursor)?;
    Ok(sound_data)
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

pub fn audio_engine_load_sound(
    engine: &mut AudioEngine,
    name: impl Into<String>,
    data: StaticSoundData,
) {
    engine.sound_cache.insert(name.into(), data);
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
    engine.spatial_filters.remove(&entity);
    #[cfg(not(target_arch = "wasm32"))]
    engine.spatial_aims.remove(&entity);
    engine.paused_handles.remove(&entity);
    #[cfg(not(target_arch = "wasm32"))]
    if let Some(stream) = engine.streams.remove(&entity) {
        stream.close();
    }
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
    decibels: f32,
    seconds: f32,
) {
    use kira::{Decibels, Tween};
    use std::time::Duration;
    let tween = Tween {
        duration: Duration::from_secs_f32(seconds.max(0.0)),
        ..Default::default()
    };
    let target = Decibels(decibels);
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

impl nightshade_load::Reader for crate::data::Sound {
    type Item = StaticSoundData;
    type Note = Option<String>;

    fn hands() -> usize {
        crate::data::LOAD_CREW
    }

    fn read(source: nightshade_load::Source) -> Result<StaticSoundData, String> {
        let bytes = nightshade_load::bytes_of(source)?;
        StaticSoundData::from_cursor(Cursor::new(bytes))
            .map_err(|error| format!("a sound did not decode: {error}"))
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn audio_engine_hold_noise(
    engine: &mut AudioEngine,
    name: impl Into<String>,
    seconds: f32,
    softness: f32,
) {
    use kira::Frame;
    use kira::sound::static_sound::StaticSoundSettings;

    let rate = 44_100u32;
    let count = ((rate as f32) * seconds.max(0.1)) as usize;
    let follow = (1.0 - softness.clamp(0.0, 0.99)).clamp(0.01, 1.0);
    let mut generator = fastrand::Rng::with_seed(0x5EED);
    let mut left = 0.0f32;
    let mut right = 0.0f32;
    let mut frames = Vec::with_capacity(count);
    for _ in 0..count {
        left += (generator.f32() * 2.0 - 1.0 - left) * follow;
        right += (generator.f32() * 2.0 - 1.0 - right) * follow;
        frames.push(Frame {
            left: left * NOISE_LEVEL,
            right: right * NOISE_LEVEL,
        });
    }

    engine.sound_cache.insert(
        name.into(),
        StaticSoundData {
            sample_rate: rate,
            frames: frames.into(),
            settings: StaticSoundSettings::default(),
            slice: None,
        },
    );
}

#[cfg(not(target_arch = "wasm32"))]
const NOISE_LEVEL: f32 = 0.5;

#[cfg(not(target_arch = "wasm32"))]
pub fn audio_stream_push(engine: &AudioEngine, entity: Entity, samples: &[f32], channels: u32) {
    if let Some(stream) = engine.streams.get(&entity) {
        stream.push_interleaved(samples, channels);
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn audio_stream_push_pcm(engine: &AudioEngine, entity: Entity, samples: &[i16], channels: u32) {
    let widened: Vec<f32> = samples
        .iter()
        .map(|sample| *sample as f32 / i16::MAX as f32)
        .collect();
    audio_stream_push(engine, entity, &widened, channels);
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(not(target_arch = "wasm32"))]
pub fn audio_stream_close(engine: &mut AudioEngine, entity: Entity) {
    if let Some(stream) = engine.streams.remove(&entity) {
        stream.close();
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn audio_capture_devices() -> Vec<String> {
    crate::data::capture::capture_devices()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn audio_capture_open(
    engine: &mut AudioEngine,
    entity: Entity,
    device: &str,
    gate: f32,
) -> bool {
    let Some(stream) = engine.streams.get(&entity).cloned() else {
        tracing::warn!("Capture asked for an entity with no live source");
        return false;
    };
    let Some(capture) = crate::data::capture::open_capture(device, stream, gate) else {
        return false;
    };
    if !capture.start() {
        return false;
    }
    engine.captures.insert(entity, capture);
    true
}

#[cfg(not(target_arch = "wasm32"))]
pub fn audio_capture_stop(engine: &AudioEngine, entity: Entity) {
    if let Some(capture) = engine.captures.get(&entity) {
        capture.stop();
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn audio_capture_close(engine: &mut AudioEngine, entity: Entity) {
    if let Some(capture) = engine.captures.remove(&entity) {
        capture.stop();
    }
}
