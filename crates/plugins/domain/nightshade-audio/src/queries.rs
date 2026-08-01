use kira::sound::static_sound::StaticSoundData;
use nightshade::ecs::Entity;

use crate::resources::AudioEngine;

pub fn audio_engine_is_initialized(engine: &AudioEngine) -> bool {
    engine.initialized && engine.manager.is_some()
}

pub fn audio_engine_get_sound<'a>(
    engine: &'a AudioEngine,
    name: &str,
) -> Option<&'a StaticSoundData> {
    engine.sound_cache.get(name)
}

pub fn audio_engine_has_handle(engine: &AudioEngine, entity: Entity) -> bool {
    #[cfg(not(target_arch = "wasm32"))]
    if engine.streams.contains_key(&entity) {
        return true;
    }
    engine.sound_handles.contains_key(&entity) || engine.spatial_tracks.contains_key(&entity)
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn bus_handle_mut(
    audio: &mut crate::resources::AudioEngine,
    bus: crate::contract::AudioBus,
) -> Option<&mut kira::track::TrackHandle> {
    match bus {
        crate::contract::AudioBus::Master => audio.buses.master.as_mut(),
        crate::contract::AudioBus::Music => audio.buses.music.as_mut(),
        crate::contract::AudioBus::Sfx => audio.buses.sfx.as_mut(),
        crate::contract::AudioBus::Ambient => audio.buses.ambient.as_mut(),
        crate::contract::AudioBus::Voice => audio.buses.voice.as_mut(),
        crate::contract::AudioBus::Ui => audio.buses.ui.as_mut(),
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn audio_stream_waiting(engine: &AudioEngine, entity: Entity) -> usize {
    engine
        .streams
        .get(&entity)
        .map(|stream| stream.waiting())
        .unwrap_or(0)
}
