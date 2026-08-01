use crate::contract::{AudioBus, AudioSource};
use crate::contract::{AudioTrack, PendingSound};
use crate::queries::{audio_engine_get_sound, audio_engine_is_initialized};
use crate::resources::AudioEngine;
use nightshade::ecs::system_param::{Commands, Query, Res};

pub fn drain_audio_requests_system(
    engine: Res<AudioEngine>,
    mut commands: Commands,
    requests: Query<(&PendingSound,)>,
) {
    let initialized = audio_engine_is_initialized(&engine);
    requests.for_each(|entity, (request,)| {
        commands.remove::<(PendingSound,)>(entity);
        if !initialized || audio_engine_get_sound(&engine, &request.clip).is_none() {
            return;
        }
        let source = AudioSource::new(request.clip.clone()).with_volume(request.volume);
        let source = match request.track {
            AudioTrack::Sound => source.with_bus(AudioBus::Sfx),
            AudioTrack::Music => source.with_looping(true).with_bus(AudioBus::Music),
        };
        commands.set(entity, source.playing());
    });
}
