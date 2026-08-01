use crate::commands::{
    audio_engine_pause_sound, audio_engine_resume_sound, audio_engine_stop_sound,
};
use crate::contract::{AudioBus, AudioListener, AudioSource};
use crate::queries::{
    audio_engine_get_sound, audio_engine_has_handle, audio_engine_is_initialized,
};
use crate::resources::AudioEngine;
use kira::{
    Decibels, Easing, Mapping, Tween, Value,
    effect::filter::FilterBuilder,
    sound::static_sound::StaticSoundData,
    track::{SpatialTrackBuilder, TrackHandle},
};
use nalgebra_glm;
use nightshade::ecs::Entity;
use nightshade::prelude::*;
use nightshade_scene::contract::GlobalTransform;
use tracing;

const OPEN_CUTOFF: f64 = 20000.0;

type KiraVec3 = mint::Vector3<f32>;
type KiraQuat = mint::Quaternion<f32>;

fn kira_position(transform: &GlobalTransform) -> KiraVec3 {
    KiraVec3 {
        x: transform.0[(0, 3)],
        y: transform.0[(1, 3)],
        z: transform.0[(2, 3)],
    }
}

fn kira_orientation(orientation: nalgebra_glm::Quat) -> KiraQuat {
    KiraQuat {
        v: mint::Vector3 {
            x: orientation.coords.x,
            y: orientation.coords.y,
            z: orientation.coords.z,
        },
        s: orientation.coords.w,
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn sync_audio_listener_system(
    mut audio: ResMut<AudioEngine>,
    listeners: Query<(&AudioListener, &GlobalTransform)>,
) {
    if !audio_engine_is_initialized(&audio) || !audio.buses_built {
        return;
    }

    let mut pose = None;
    let mut place = nalgebra_glm::Vec3::zeros();
    let mut turn = nalgebra_glm::Quat::identity();
    listeners.for_each(|_entity, (_listener, transform)| {
        if pose.is_none() {
            turn = nalgebra_glm::mat3_to_quat(&upright(&transform.0));
            place = nalgebra_glm::Vec3::new(
                transform.0[(0, 3)],
                transform.0[(1, 3)],
                transform.0[(2, 3)],
            );
            pose = Some((kira_position(transform), kira_orientation(turn)));
        }
    });
    audio.listener_place = place;
    audio.listener_turn = turn;

    let Some((position, orientation)) = pose else {
        return;
    };

    if audio.listener.is_none() {
        let listener_result = audio.manager.as_mut().map(|manager| {
            manager.add_listener(position, kira_orientation(nalgebra_glm::Quat::identity()))
        });
        match listener_result {
            Some(Ok(listener)) => {
                audio.listener = Some(listener);
                tracing::info!("Created audio listener");
            }
            Some(Err(error)) => {
                tracing::error!("Failed to create audio listener: {}", error);
                return;
            }
            None => return,
        }
    }

    if let Some(listener_handle) = &mut audio.listener {
        listener_handle.set_position(position, Tween::default());
        listener_handle.set_orientation(orientation, Tween::default());
    }
}

enum Voice {
    Clip(Box<StaticSoundData>),
    Live(u32),
}

struct PlayIntent {
    entity: Entity,
    voice: Voice,
    is_spatial: bool,
    bus: AudioBus,
    min_distance: f32,
    max_distance: f32,
    rolloff: f32,
    reverb_zones: Vec<(String, f32)>,
    position: KiraVec3,
}

#[cfg(not(target_arch = "wasm32"))]
pub fn update_audio_sources_system(
    mut audio: ResMut<AudioEngine>,
    time: Res<nightshade_platform::contract::time::Time>,
    sources: Query<(&AudioSource, Option<&GlobalTransform>)>,
    mut source_lookup: Query<(&mut AudioSource,)>,
) {
    if !audio_engine_is_initialized(&audio) || !audio.buses_built {
        return;
    }

    let uptime_milliseconds = time.uptime_milliseconds;
    let mut sounds_to_play: Vec<PlayIntent> = Vec::new();
    let mut picked: Vec<(Entity, String)> = Vec::new();
    let last_clips = audio.last_clips.clone();
    let mut sounds_to_stop = Vec::new();
    let mut spatial_tracks_to_update = Vec::new();

    sources.for_each(|entity, (source, transform)| {
        let has_handle = audio_engine_has_handle(&audio, entity);
        let position = transform.map(kira_position).unwrap_or(KiraVec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        });

        if source.playing && !has_handle && source.live {
            sounds_to_play.push(PlayIntent {
                entity,
                voice: Voice::Live(source.live_rate),
                is_spatial: source.spatial,
                bus: source.bus,
                min_distance: source.min_distance,
                max_distance: source.max_distance,
                rolloff: source.rolloff,
                reverb_zones: source.reverb_zones.clone(),
                position: if source.spatial {
                    position
                } else {
                    KiraVec3 {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    }
                },
            });
        } else if source.playing
            && !has_handle
            && let Some(audio_ref) =
                pick_active_clip(source, entity, uptime_milliseconds, last_clips.get(&entity))
            && let Some(sound_data) = audio_engine_get_sound(&audio, &audio_ref)
        {
            let mut sound_data = sound_data.clone();
            if source.looping {
                sound_data = sound_data.loop_region(..);
            }
            let wander = wobble(entity, uptime_milliseconds);
            let swing = source.volume_spread * (wander * 2.0 - 1.0);
            sound_data = sound_data.volume(Decibels(Decibels::from(source.volume).0 + swing));
            let spread = source.pitch_spread.max(1.0);
            let bent = source.playback_rate
                * (1.0 / spread as f64 + (spread as f64 - 1.0 / spread as f64) * wander as f64);
            if (bent - 1.0).abs() > f64::EPSILON {
                sound_data = sound_data.playback_rate(bent);
            }
            picked.push((entity, audio_ref.clone()));

            sounds_to_play.push(PlayIntent {
                entity,
                voice: Voice::Clip(Box::new(sound_data)),
                is_spatial: source.spatial,
                bus: source.bus,
                min_distance: source.min_distance,
                max_distance: source.max_distance,
                rolloff: source.rolloff,
                reverb_zones: source.reverb_zones.clone(),
                position: if source.spatial {
                    position
                } else {
                    KiraVec3 {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    }
                },
            });
        } else if !source.playing && has_handle {
            sounds_to_stop.push(entity);
        }

        if source.spatial && transform.is_some() && audio.spatial_tracks.contains_key(&entity) {
            spatial_tracks_to_update.push((entity, position));
        }
    });

    let mut failed_to_play = Vec::new();
    for intent in sounds_to_play {
        start_voice(&mut audio, intent, &mut failed_to_play);
    }

    for entity in failed_to_play {
        source_lookup.for_entity(entity, |(source,)| source.playing = false);
    }

    #[cfg(not(target_arch = "wasm32"))]
    let heard_from = audio.listener_place;
    #[cfg(not(target_arch = "wasm32"))]
    let heard_turn = nalgebra_glm::quat_inverse(&audio.listener_turn);
    for (entity, position) in spatial_tracks_to_update {
        if let Some(spatial_track) = audio.spatial_tracks.get_mut(&entity) {
            spatial_track.set_position(position, Tween::default());
        }
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(aim) = audio.spatial_aims.get(&entity) {
            let toward = nalgebra_glm::Vec3::new(position.x, position.y, position.z) - heard_from;
            let local = nalgebra_glm::quat_rotate_vec3(&heard_turn, &toward);
            aim.set_aim(local.x.atan2(-local.z));
        }
    }

    for (entity, clip) in picked {
        audio.last_clips.insert(entity, clip);
    }

    for entity in sounds_to_stop {
        audio_engine_stop_sound(&mut audio, entity);
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn start_voice(audio: &mut AudioEngine, intent: PlayIntent, failed_to_play: &mut Vec<Entity>) {
    if intent.is_spatial && audio.listener.is_none() {
        tracing::warn!("Spatial audio requested but no listener exists");
        return;
    }

    if intent.is_spatial {
        let listener = audio.listener.as_ref().unwrap();
        let reverb_sends = &audio.reverb_sends;
        let distance_input = (intent.min_distance as f64, intent.max_distance as f64);
        let mut track_builder = SpatialTrackBuilder::new()
            .distances((intent.min_distance, intent.max_distance))
            .attenuation_function(Easing::OutPowf(intent.rolloff.max(0.1) as f64));
        #[cfg(not(target_arch = "wasm32"))]
        {
            track_builder = track_builder.spatialization_strength(0.0);
        }

        for (zone_name, send_decibels) in &intent.reverb_zones {
            if let Some(send) = reverb_sends.get(zone_name) {
                track_builder = track_builder.with_send(
                    send,
                    Value::FromListenerDistance(Mapping {
                        input_range: distance_input,
                        output_range: (Decibels(send_decibels - 6.0), Decibels(*send_decibels)),
                        easing: Easing::OutPowf(intent.rolloff.max(0.1) as f64),
                    }),
                );
            } else {
                tracing::warn!("Unknown reverb zone '{}'", zone_name);
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        for index in 0..crate::systems::buses::ZONE_SENDS {
            if let Some(send) = reverb_sends.get(&format!("zone_{index}")) {
                track_builder = track_builder.with_send(send, Decibels::SILENCE);
            }
        }

        let filter = track_builder.add_effect(FilterBuilder::new().cutoff(OPEN_CUTOFF));
        #[cfg(not(target_arch = "wasm32"))]
        let aim = track_builder.add_effect(crate::data::binaural::BinauralBuilder::new());

        let bus_slot: &mut Option<TrackHandle> = match intent.bus {
            AudioBus::Master => &mut audio.buses.master,
            AudioBus::Music => &mut audio.buses.music,
            AudioBus::Sfx => &mut audio.buses.sfx,
            AudioBus::Ambient => &mut audio.buses.ambient,
            AudioBus::Voice => &mut audio.buses.voice,
            AudioBus::Ui => &mut audio.buses.ui,
        };
        let Some(bus) = bus_slot.as_mut() else {
            tracing::warn!("Audio bus not built for entity {:?}", intent.entity);
            return;
        };
        match bus.add_spatial_sub_track(listener, intent.position, track_builder) {
            Ok(mut spatial_track) => {
                let started = match intent.voice {
                    Voice::Clip(sound_data) => match spatial_track.play(*sound_data) {
                        Ok(sound_handle) => {
                            audio
                                .spatial_sound_handles
                                .insert(intent.entity, sound_handle);
                            true
                        }
                        Err(error) => {
                            tracing::error!(
                                "Failed to play sound on spatial track for entity {:?}: {}",
                                intent.entity,
                                error
                            );
                            false
                        }
                    },
                    Voice::Live(rate) => {
                        let live = crate::data::stream::StreamSoundData::new(rate);
                        let handle = live.handle();
                        match spatial_track.play(live) {
                            Ok(_) => {
                                audio.streams.insert(intent.entity, handle);
                                true
                            }
                            Err(error) => {
                                tracing::error!(
                                    "Failed to open live stream for entity {:?}: {}",
                                    intent.entity,
                                    error
                                );
                                false
                            }
                        }
                    }
                };
                if started {
                    audio.spatial_tracks.insert(intent.entity, spatial_track);
                    audio.spatial_filters.insert(intent.entity, filter);
                    #[cfg(not(target_arch = "wasm32"))]
                    audio.spatial_aims.insert(intent.entity, aim);
                }
            }
            Err(error) => {
                tracing::error!(
                    "Failed to create spatial track for entity {:?}: {}",
                    intent.entity,
                    error
                );
            }
        }
    } else {
        let bus_slot: &mut Option<TrackHandle> = match intent.bus {
            AudioBus::Master => &mut audio.buses.master,
            AudioBus::Music => &mut audio.buses.music,
            AudioBus::Sfx => &mut audio.buses.sfx,
            AudioBus::Ambient => &mut audio.buses.ambient,
            AudioBus::Voice => &mut audio.buses.voice,
            AudioBus::Ui => &mut audio.buses.ui,
        };
        let Some(bus) = bus_slot.as_mut() else {
            tracing::warn!("Audio bus not built for entity {:?}", intent.entity);
            return;
        };
        match intent.voice {
            Voice::Clip(sound_data) => match bus.play(*sound_data) {
                Ok(handle) => {
                    audio.sound_handles.insert(intent.entity, handle);
                }
                Err(error) => {
                    tracing::error!(
                        "Failed to play sound for entity {:?}: {}",
                        intent.entity,
                        error
                    );
                    failed_to_play.push(intent.entity);
                }
            },
            Voice::Live(rate) => {
                let live = crate::data::stream::StreamSoundData::new(rate);
                let handle = live.handle();
                match bus.play(live) {
                    Ok(_) => {
                        audio.streams.insert(intent.entity, handle);
                    }
                    Err(error) => {
                        tracing::error!(
                            "Failed to open live stream for entity {:?}: {}",
                            intent.entity,
                            error
                        );
                        failed_to_play.push(intent.entity);
                    }
                }
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn sync_audio_pause_system(mut audio: ResMut<AudioEngine>, sources: Query<(&AudioSource,)>) {
    if !audio_engine_is_initialized(&audio) || !audio.buses_built {
        return;
    }

    let mut to_pause = Vec::new();
    let mut to_resume = Vec::new();
    sources.for_each(|entity, (source,)| {
        if !audio_engine_has_handle(&audio, entity) {
            return;
        }
        let wants_pause = source.paused && source.playing;
        let currently_paused = audio.paused_handles.contains(&entity);
        if wants_pause && !currently_paused {
            to_pause.push(entity);
        } else if !wants_pause && currently_paused {
            to_resume.push(entity);
        }
    });

    for entity in to_pause {
        audio_engine_pause_sound(&mut audio, entity);
        audio.paused_handles.insert(entity);
    }
    for entity in to_resume {
        audio_engine_resume_sound(&mut audio, entity);
        audio.paused_handles.remove(&entity);
    }
}

fn wobble(entity: Entity, uptime_milliseconds: u64) -> f32 {
    let mut seed = uptime_milliseconds.wrapping_mul(0x9E3779B97F4A7C15);
    seed ^= (entity.id as u64).wrapping_mul(0xD6E8FEB86659FD93);
    seed ^= seed >> 32;
    (seed & 0xFFFFFF) as f32 / 0xFFFFFF as f32
}

fn pick_active_clip(
    source: &AudioSource,
    entity: Entity,
    uptime_milliseconds: u64,
    last: Option<&String>,
) -> Option<String> {
    if !source.random_pick || source.random_clips.is_empty() {
        return source.audio_ref.clone();
    }
    let mut pool: Vec<&String> = Vec::with_capacity(source.random_clips.len() + 1);
    if let Some(reference) = source.audio_ref.as_ref() {
        pool.push(reference);
    }
    for clip in &source.random_clips {
        if !clip.is_empty() {
            pool.push(clip);
        }
    }
    if source.no_repeat
        && pool.len() > 1
        && let Some(held) = last
    {
        pool.retain(|clip| *clip != held);
    }
    if pool.is_empty() {
        return None;
    }
    let mut seed = uptime_milliseconds.wrapping_mul(0x9E3779B97F4A7C15);
    seed ^= (entity.id as u64).wrapping_mul(0xBF58476D1CE4E5B9);
    seed = seed.wrapping_mul(0x94D049BB133111EB);
    let index = ((seed >> 32) as usize) % pool.len();
    Some(pool[index].clone())
}

#[cfg(not(target_arch = "wasm32"))]
fn upright(transform: &nalgebra_glm::Mat4) -> nalgebra_glm::Mat3 {
    let basis = nalgebra_glm::mat4_to_mat3(transform);
    let right = straighten(basis.column(0).into());
    let up = straighten(basis.column(1).into());
    let ahead = straighten(basis.column(2).into());
    nalgebra_glm::Mat3::from_columns(&[right, up, ahead])
}

#[cfg(not(target_arch = "wasm32"))]
fn straighten(axis: nalgebra_glm::Vec3) -> nalgebra_glm::Vec3 {
    let length = axis.magnitude();
    if length > 1.0e-6 { axis / length } else { axis }
}
