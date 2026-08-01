use crate::data::{CLIP, RAIN, RAIN_LOUDEST, RAIN_QUIETEST, RAIN_REACH, ROOM_TONE};
use crate::resources::AcousticsDemo;
use nightshade::prelude::*;
use nightshade_acoustics::contract::{AcousticEmitter, AcousticsFrame, AcousticsRoom};
use nightshade_audio::commands::{
    audio_engine_fade_volume, audio_engine_hold_noise, audio_engine_load_sound_from_bytes,
};
use nightshade_audio::contract::AudioSource;
use nightshade_audio::queries::audio_engine_is_initialized;
use nightshade_audio::resources::AudioEngine;
use nightshade_scene::contract::LocalTransform;

pub fn start_the_tone(
    mut audio: ResMut<AudioEngine>,
    mut demo: ResMut<AcousticsDemo>,
    sources: Query<(&mut AudioSource,)>,
) {
    if demo.playing || !audio_engine_is_initialized(&audio) {
        return;
    }
    if !audio_engine_load_sound_from_bytes(&mut audio, CLIP, ROOM_TONE.to_vec()) {
        return;
    }
    audio_engine_hold_noise(&mut audio, RAIN, 4.0, 0.72);
    sources.for_each(|_entity, (source,)| {
        source.playing = true;
    });
    demo.playing = true;
}

pub fn mark_the_sounding(voices: Query<(&AudioSource, &mut AcousticEmitter)>) {
    voices.for_each(|_entity, (source, emitter)| {
        emitter.sounding = source.playing && !source.paused;
    });
}

pub fn steer_the_rain(
    frame: Res<AcousticsFrame>,
    room: Res<AcousticsRoom>,
    demo: Res<AcousticsDemo>,
    mut audio: ResMut<AudioEngine>,
    mut places: Query<(&mut LocalTransform,)>,
) {
    if !demo.built || !frame.heard {
        return;
    }

    let outside = room.heard.outdoor_low.clamp(0.0, 1.0);
    let place = frame.listener + room.heard.opening * RAIN_REACH;
    places.for_entity(demo.rain, |(transform,)| {
        transform.translation = place;
    });

    let level = RAIN_QUIETEST + (RAIN_LOUDEST - RAIN_QUIETEST) * outside;
    audio_engine_fade_volume(&mut audio, demo.rain, level, 0.15);
}
