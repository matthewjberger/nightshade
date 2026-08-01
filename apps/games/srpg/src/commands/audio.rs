use crate::theme;
use fastrand::Rng;
use nightshade::prelude::*;
use nightshade_audio::commands::audio_engine_load_sound_from_bytes;
use nightshade_audio::contract::{AudioBus, AudioSource};
use nightshade_audio::resources::AudioEngine;
use nightshade_scene::prelude::Lifetime;

fn wav(samples: &[f32]) -> Vec<u8> {
    let rate = theme::SOUND_RATE;
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

fn seconds(count: f32) -> usize {
    (theme::SOUND_RATE as f32 * count) as usize
}

fn envelope(index: usize, total: usize, attack: f32) -> f32 {
    let share = index as f32 / total as f32;
    let rise = (share / attack).min(1.0);
    rise * (1.0 - share).powf(1.6)
}

fn tone(length: f32, from: f32, to: f32, attack: f32, shape: f32) -> Vec<f32> {
    let total = seconds(length);
    (0..total)
        .map(|index| {
            let share = index as f32 / total as f32;
            let pitch = from + (to - from) * share;
            let phase = index as f32 / theme::SOUND_RATE as f32 * pitch * std::f32::consts::TAU;
            let wave = phase.sin() * (1.0 - shape) + (phase.sin().signum()) * shape;
            wave * envelope(index, total, attack) * 0.5
        })
        .collect()
}

fn thud(length: f32, from: f32, to: f32, grit: f32) -> Vec<f32> {
    let total = seconds(length);
    let mut rng = Rng::with_seed(7_u64);
    (0..total)
        .map(|index| {
            let share = index as f32 / total as f32;
            let pitch = from + (to - from) * share;
            let phase = index as f32 / theme::SOUND_RATE as f32 * pitch * std::f32::consts::TAU;
            let noise = rng.u32(..) as f32 / u32::MAX as f32 * 2.0 - 1.0;
            (phase.sin() * (1.0 - grit) + noise * grit) * envelope(index, total, 0.01) * 0.6
        })
        .collect()
}

fn chord(length: f32, steps: &[f32]) -> Vec<f32> {
    let each = length / steps.len() as f32;
    steps
        .iter()
        .flat_map(|pitch| tone(each, *pitch, *pitch, 0.05, 0.15))
        .collect()
}

fn bed(length: f32, root: f32, steps: &[f32], pulse: f32) -> Vec<f32> {
    let total = seconds(length);
    (0..total)
        .map(|index| {
            let clock = index as f32 / theme::SOUND_RATE as f32;
            let share = clock / length;
            let step = steps[(share * steps.len() as f32) as usize % steps.len()];
            let low = (clock * root * std::f32::consts::TAU).sin() * 0.20;
            let lead = (clock * root * step * std::f32::consts::TAU).sin() * 0.12;
            let swell = (share * std::f32::consts::TAU).sin() * 0.5 + 0.5;
            let beat = ((clock * pulse).fract() * -6.0).exp();
            (low + lead * swell) * (0.6 + beat * 0.4)
        })
        .collect()
}

pub fn load_the_sounds(engine: &mut AudioEngine) {
    let clips: [(&str, Vec<f32>); 10] = [
        (theme::SOUND_STEP, tone(0.07, 300.0, 240.0, 0.02, 0.1)),
        (theme::SOUND_PICK, tone(0.06, 680.0, 680.0, 0.02, 0.4)),
        (theme::SOUND_CONFIRM, chord(0.16, &[560.0, 840.0])),
        (theme::SOUND_HIT, thud(0.20, 190.0, 70.0, 0.55)),
        (theme::SOUND_HEAL, tone(0.34, 480.0, 760.0, 0.15, 0.0)),
        (theme::SOUND_DOWN, thud(0.60, 260.0, 60.0, 0.35)),
        (theme::SOUND_CLAIM, chord(0.45, &[440.0, 554.0, 660.0])),
        (
            theme::SOUND_WIN,
            chord(0.90, &[523.0, 659.0, 784.0, 1046.0]),
        ),
        (theme::SOUND_LOSS, chord(0.90, &[392.0, 349.0, 294.0])),
        (
            theme::SOUND_CAMP,
            bed(9.0, 98.0, &[3.0, 4.0, 3.0, 2.5], 0.5),
        ),
    ];
    for (name, samples) in clips {
        audio_engine_load_sound_from_bytes(engine, name, wav(&samples));
    }
    let battle = bed(8.0, 73.0, &[2.0, 3.0, 4.0, 3.0], 2.0);
    audio_engine_load_sound_from_bytes(engine, theme::SOUND_BATTLE, wav(&battle));
}

pub fn play(commands: &mut Commands, clip: &'static str, volume: f32) {
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
    commands.set(entity, Lifetime { remaining: 4.0 });
}

pub fn open_the_track(commands: &mut Commands, clip: &'static str, volume: f32) -> Entity {
    let entity = commands.spawn(());
    commands.set(
        entity,
        AudioSource {
            audio_ref: Some(clip.to_string()),
            volume,
            playing: false,
            looping: true,
            bus: AudioBus::Music,
            ..Default::default()
        },
    );
    entity
}
