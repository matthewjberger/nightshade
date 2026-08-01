use crate::data::{
    DRUM_LEVEL, MUSIC_LEVEL, SOUND_BATTLE_BED, SOUND_FIREBALL, SOUND_RATE, SOUND_SPRINT,
    SOUND_WHIRL,
};
use crate::resources::{Cue, Cues};
use nightshade::prelude::*;
use nightshade_audio::commands::audio_engine_load_sound_from_bytes;
use nightshade_audio::contract::{AudioBus, AudioSource};
use nightshade_audio::resources::AudioEngine;
use nightshade_kit::prelude::CLIPS;
use nightshade_random::prelude::*;
use nightshade_scene::prelude::Lifetime;

const TAU: f32 = std::f32::consts::TAU;

fn wav(samples: &[f32]) -> Vec<u8> {
    let rate = SOUND_RATE;
    let data = samples.len() as u32 * 2;
    let mut bytes = Vec::with_capacity(44 + data as usize);
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&(36 + data).to_le_bytes());
    bytes.extend_from_slice(b"WAVEfmt ");
    bytes.extend_from_slice(&16u32.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&rate.to_le_bytes());
    bytes.extend_from_slice(&(rate * 2).to_le_bytes());
    bytes.extend_from_slice(&2u16.to_le_bytes());
    bytes.extend_from_slice(&16u16.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&data.to_le_bytes());
    for sample in samples {
        let value = (sample.clamp(-1.0, 1.0) * 30000.0) as i16;
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    bytes
}

fn envelope(index: usize, total: usize, attack: f32) -> f32 {
    let share = index as f32 / total as f32;
    let rise = (share / attack).min(1.0);
    rise * (1.0 - share).powf(1.6)
}

fn whoosh(length: f32, from: f32, to: f32) -> Vec<f32> {
    let total = (SOUND_RATE as f32 * length) as usize;
    let mut random = Random::seeded(11);
    let mut low = 0.0f32;
    let mut band = 0.0f32;
    (0..total)
        .map(|index| {
            let share = index as f32 / total as f32;
            let cutoff = from + (to - from) * share;
            let coefficient = (cutoff * TAU / SOUND_RATE as f32).min(0.9);
            let noise = random.f32() * 2.0 - 1.0;
            low += (noise - low) * coefficient;
            band += (low - band) * coefficient;
            band * envelope(index, total, 0.25) * 2.4
        })
        .collect()
}

fn tone(length: f32, from: f32, to: f32) -> Vec<f32> {
    let total = (SOUND_RATE as f32 * length) as usize;
    let mut phase = 0.0f32;
    (0..total)
        .map(|index| {
            let share = index as f32 / total as f32;
            let pitch = from + (to - from) * share;
            phase += pitch * TAU / SOUND_RATE as f32;
            (phase.sin() * 0.7 + phase.sin().signum() * 0.3) * envelope(index, total, 0.2) * 0.4
        })
        .collect()
}

fn mix(first: &[f32], second: &[f32]) -> Vec<f32> {
    let length = first.len().max(second.len());
    (0..length)
        .map(|index| {
            first.get(index).copied().unwrap_or(0.0) + second.get(index).copied().unwrap_or(0.0)
        })
        .collect()
}

pub fn load_clips(engine: &mut AudioEngine) {
    for (name, bytes) in CLIPS {
        audio_engine_load_sound_from_bytes(engine, name, bytes.to_vec());
    }
    let made: [(&str, Vec<f32>); 3] = [
        (
            SOUND_FIREBALL,
            mix(&whoosh(0.5, 300.0, 1800.0), &tone(0.45, 180.0, 420.0)),
        ),
        (
            SOUND_WHIRL,
            mix(&whoosh(0.65, 250.0, 1400.0), &whoosh(0.65, 1400.0, 250.0)),
        ),
        (SOUND_SPRINT, whoosh(0.35, 200.0, 1600.0)),
    ];
    for (name, samples) in made {
        audio_engine_load_sound_from_bytes(engine, name, wav(&samples));
    }
}

pub fn cue(cues: &mut Cues, clip: &'static str, volume: f32) {
    cues.pending.push(Cue { clip, volume });
}

pub fn cue_one_of(cues: &mut Cues, random: &mut Random, clips: &[&'static str], volume: f32) {
    cue(cues, clips[random.usize(0..clips.len())], volume);
}

pub fn play(commands: &mut Commands, clip: &str, volume: f32) {
    let entity = commands.spawn(());
    commands.set(
        entity,
        AudioSource {
            audio_ref: Some(clip.to_string()),
            volume,
            playing: true,
            bus: AudioBus::Sfx,
            ..Default::default()
        },
    );
    commands.set(entity, Lifetime { remaining: 8.0 });
}

pub fn open_track(commands: &mut Commands, clip: &'static str, playing: bool) -> Entity {
    let entity = commands.spawn(());
    let volume = if clip == SOUND_BATTLE_BED {
        DRUM_LEVEL
    } else {
        MUSIC_LEVEL
    };
    commands.set(
        entity,
        AudioSource {
            audio_ref: Some(clip.to_string()),
            volume,
            playing,
            looping: true,
            bus: AudioBus::Music,
            ..Default::default()
        },
    );
    entity
}
