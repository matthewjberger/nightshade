use crate::resources::AudioEngine;
use kira::{Decibels, Tween};
use nightshade::prelude::*;
use nightshade_acoustics::contract::{
    AcousticsHeard, AcousticsRoom, AcousticsSettings, RoomReading,
};
use std::time::Duration;

const SETTLE_MILLISECONDS: u64 = 80;
const QUIETEST_ROOM: f32 = -48.0;
const QUIETEST_PATH: f32 = -60.0;
const WIDEST_PAN: f32 = 0.7;

pub fn drive_audio_acoustics_system(
    mut audio: ResMut<AudioEngine>,
    heard: Option<Res<AcousticsHeard>>,
    room: Option<Res<AcousticsRoom>>,
    settings: Option<Res<AcousticsSettings>>,
) {
    if !audio.buses_built {
        return;
    }
    let (Some(heard), Some(room), Some(settings)) = (heard, room, settings) else {
        return;
    };
    let settle = Tween {
        duration: Duration::from_millis(SETTLE_MILLISECONDS),
        ..Default::default()
    };

    let listener_send = audio.reverb_sends.get("default").map(|send| send.id());
    for (entity, path) in &heard.paths {
        if let Some(filter) = audio.spatial_filters.get_mut(entity) {
            filter.set_cutoff(
                path.cutoff(settings.low_band, settings.high_band) as f64,
                settle,
            );
        }
        let reaching = if settings.reverb_only {
            0.0
        } else {
            path.gain_low
        };
        if let Some(track) = audio.spatial_tracks.get_mut(entity) {
            track.set_volume(Decibels(quieted(reaching)), settle);
        }
        if !path.listener_reverb
            && let Some(send) = listener_send
            && let Some(track) = audio.spatial_tracks.get_mut(entity)
        {
            let _ = track.set_send(send, Decibels::SILENCE, settle);
        }
    }

    let heard_turn = nalgebra_glm::quat_inverse(&audio.listener_turn);
    if let Some(reverb) = audio.room_reverb.as_ref() {
        shape_reverb(reverb, &room.heard, &heard_turn, &settings);
    }

    let sends: Vec<(usize, kira::track::SendTrackId)> = (0..audio.zone_reverbs.len())
        .filter_map(|index| {
            audio
                .reverb_sends
                .get(&format!("zone_{index}"))
                .map(|send| (index, send.id()))
        })
        .collect();

    for (index, (entity, reading)) in room.zones.iter().enumerate() {
        let Some(reverb) = audio.zone_reverbs.get(index) else {
            break;
        };
        shape_reverb(reverb, reading, &heard_turn, &settings);
        let carried = heard
            .paths
            .iter()
            .find(|(held, _)| held == entity)
            .map(|(_, path)| path.gain_low)
            .unwrap_or(0.0);
        let alike = likeness(&room.heard, reading);
        let apart = settings.relative_outer - settings.relative_inner;
        let blend = 1.0 - ((alike - settings.relative_inner) / apart.max(1.0e-3)).clamp(0.0, 1.0);
        let Some((_, send)) = sends.iter().find(|(slot, _)| *slot == index) else {
            continue;
        };
        if let Some(track) = audio.spatial_tracks.get_mut(entity) {
            let _ = track.set_send(*send, Decibels(quieted(carried * blend)), settle);
        }
    }

    if let Some(level) = audio.room_level.as_mut() {
        let loudness = room.heard.loudness.clamp(0.0, 1.0);
        level.set_volume(Decibels(QUIETEST_ROOM * (1.0 - loudness)), settle);
    }
}

fn shape_reverb(
    reverb: &crate::data::room_reverb::RoomReverbHandle,
    reading: &RoomReading,
    heard_turn: &nalgebra_glm::Quat,
    settings: &AcousticsSettings,
) {
    use crate::data::room_reverb as room_settings;

    let indoors = (1.0 - reading.outdoors).clamp(0.0, 1.0);
    let (early_left, early_right) = sided(heard_turn, reading.early_from);
    let (late_left, late_right) = sided(heard_turn, reading.late_from);

    reverb.set(room_settings::GAIN, 1.0);
    reverb.set(room_settings::GAIN_LOW, 1.0);
    reverb.set(
        room_settings::GAIN_HIGH,
        (1.0 - reading.absorbed).clamp(0.05, 1.0),
    );
    reverb.set(room_settings::DECAY, reading.decay.max(0.05));
    reverb.set(room_settings::DECAY_LOW_RATIO, 1.0);
    reverb.set(
        room_settings::DECAY_HIGH_RATIO,
        (reading.decay_high / reading.decay.max(0.05)).clamp(0.1, 1.0),
    );
    reverb.set(room_settings::DIFFUSION, reading.scattered.clamp(0.1, 1.0));
    reverb.set(room_settings::DENSITY, reading.loudness.clamp(0.25, 1.0));
    reverb.set(room_settings::EARLY_GAIN, reading.early_gain);
    reverb.set(room_settings::EARLY_DELAY, reading.early_delay);
    reverb.set(room_settings::EARLY_LEFT, early_left);
    reverb.set(room_settings::EARLY_RIGHT, early_right);
    reverb.set(room_settings::LATE_GAIN, reading.late_gain.max(0.05));
    reverb.set(room_settings::LATE_DELAY, reading.late_delay);
    reverb.set(room_settings::LATE_LEFT, late_left);
    reverb.set(room_settings::LATE_RIGHT, late_right);
    reverb.set(
        room_settings::ECHO_DEPTH,
        (1.0 - reading.scattered) * indoors,
    );
    reverb.set(room_settings::SWAY_DEPTH, reading.scattered * 0.5);
    reverb.set(room_settings::AIR_HIGH, 0.994);
    reverb.set(room_settings::LOW_HERTZ, settings.low_band);
    reverb.set(room_settings::HIGH_HERTZ, settings.high_band);
}

fn likeness(heard: &RoomReading, zone: &RoomReading) -> f32 {
    let decay = (heard.decay - zone.decay).abs() / heard.decay.max(zone.decay).max(0.05);
    let tone = (heard.absorbed - zone.absorbed).abs();
    let open = (heard.outdoors - zone.outdoors).abs();
    (1.0 - (decay + tone + open) / 3.0).clamp(0.0, 1.0)
}

fn sided(heard_turn: &nalgebra_glm::Quat, from: nalgebra_glm::Vec3) -> (f32, f32) {
    if from.magnitude() < 1.0e-3 {
        return (1.0, 1.0);
    }
    let local = nalgebra_glm::quat_rotate_vec3(heard_turn, &from);
    let aside = local.x.clamp(-1.0, 1.0) * WIDEST_PAN;
    ((1.0 - aside).clamp(0.0, 2.0), (1.0 + aside).clamp(0.0, 2.0))
}

fn quieted(gain: f32) -> f32 {
    if gain <= 0.001 {
        return QUIETEST_PATH;
    }
    (20.0 * gain.log10()).clamp(QUIETEST_PATH, 0.0)
}
