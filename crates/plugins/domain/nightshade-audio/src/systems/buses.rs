use crate::resources::AudioEngine;
use kira::{
    Decibels, Easing, Mapping, Value,
    effect::volume_control::VolumeControlBuilder,
    modulator::tweener::TweenerBuilder,
    track::{SendTrackBuilder, TrackBuilder},
};
use nightshade::prelude::*;
use tracing;

const VOICE_DUCKING_DEPTH_DB: f32 = -18.0;
pub const ZONE_SENDS: usize = 4;

#[cfg(not(target_arch = "wasm32"))]
pub fn build_audio_buses_system(mut audio: ResMut<AudioEngine>) {
    if audio.buses_built {
        return;
    }
    let Some(manager) = audio.manager.as_mut() else {
        return;
    };

    let voice_ducking = match manager.add_modulator(TweenerBuilder { initial_value: 0.0 }) {
        Ok(handle) => handle,
        Err(error) => {
            tracing::error!("Failed to create voice ducking tweener: {}", error);
            return;
        }
    };
    let voice_ducking_id = voice_ducking.id();

    let ducking_mapping = Mapping {
        input_range: (0.0, 1.0),
        output_range: (Decibels::IDENTITY, Decibels(VOICE_DUCKING_DEPTH_DB)),
        easing: Easing::Linear,
    };

    let room_reverb;
    let room_level;
    let default_reverb = match manager.add_send_track({
        let mut builder = SendTrackBuilder::new();
        room_reverb = builder.add_effect(crate::data::room_reverb::RoomReverbBuilder::new());
        room_level = builder.add_effect(VolumeControlBuilder::new(Decibels::IDENTITY));
        builder
    }) {
        Ok(handle) => handle,
        Err(error) => {
            tracing::error!("Failed to create default reverb send: {}", error);
            return;
        }
    };

    let mut zone_reverbs = Vec::new();
    let mut zone_sends = Vec::new();
    for _ in 0..ZONE_SENDS {
        let handle;
        match manager.add_send_track({
            let mut builder = SendTrackBuilder::new();
            handle = builder.add_effect(crate::data::room_reverb::RoomReverbBuilder::new());
            builder
        }) {
            Ok(send) => {
                zone_reverbs.push(handle);
                zone_sends.push(send);
            }
            Err(error) => {
                tracing::error!("Failed to create zone reverb send: {}", error);
                break;
            }
        }
    }

    let mut master = match manager.add_sub_track(TrackBuilder::new()) {
        Ok(handle) => handle,
        Err(error) => {
            tracing::error!("Failed to create master bus: {}", error);
            return;
        }
    };

    let music = master.add_sub_track(TrackBuilder::new().with_effect(VolumeControlBuilder::new(
        Value::FromModulator {
            id: voice_ducking_id,
            mapping: ducking_mapping,
        },
    )));
    let ambient = master.add_sub_track(TrackBuilder::new().with_effect(VolumeControlBuilder::new(
        Value::FromModulator {
            id: voice_ducking_id,
            mapping: ducking_mapping,
        },
    )));
    let sfx = master.add_sub_track(TrackBuilder::new());
    let voice = master.add_sub_track(TrackBuilder::new());
    let ui = master.add_sub_track(TrackBuilder::new());

    let (music, ambient, sfx, voice, ui) = match (music, ambient, sfx, voice, ui) {
        (Ok(music), Ok(ambient), Ok(sfx), Ok(voice), Ok(ui)) => (music, ambient, sfx, voice, ui),
        _ => {
            tracing::error!("Failed to create one or more audio buses");
            return;
        }
    };

    audio.buses.master = Some(master);
    audio.buses.music = Some(music);
    audio.buses.ambient = Some(ambient);
    audio.buses.sfx = Some(sfx);
    audio.buses.voice = Some(voice);
    audio.buses.ui = Some(ui);
    audio.voice_ducking = Some(voice_ducking);
    audio.room_reverb = Some(room_reverb);
    audio.zone_reverbs = zone_reverbs;
    for (index, send) in zone_sends.into_iter().enumerate() {
        audio.reverb_sends.insert(format!("zone_{index}"), send);
    }
    audio.room_level = Some(room_level);
    audio
        .reverb_sends
        .insert("default".to_string(), default_reverb);
    audio.buses_built = true;
    tracing::info!("Audio buses built");
}
