use crate::ecs::world::CORE;
use crate::ecs::world::{Entity, World};
use crate::plugins::audio::components::{AudioBus, AudioSource};
use crate::plugins::audio::resources::{
    AudioEngine, audio_engine_get_sound, audio_engine_has_handle, audio_engine_initialize,
    audio_engine_is_initialized, audio_engine_pause_sound, audio_engine_resume_sound,
    audio_engine_stop_sound,
};
use kira::{
    Decibels, Easing, Mapping, Mix, Tween, Value,
    effect::{filter::FilterBuilder, reverb::ReverbBuilder, volume_control::VolumeControlBuilder},
    modulator::tweener::TweenerBuilder,
    sound::static_sound::StaticSoundData,
    track::{SendTrackBuilder, SpatialTrackBuilder, TrackBuilder, TrackHandle},
};
use std::io::Cursor;
use std::time::Duration;

type KiraVec3 = mint::Vector3<f32>;
type KiraQuat = mint::Quaternion<f32>;

const VOICE_DUCKING_DEPTH_DB: f32 = -18.0;

/// Opens the audio device. `AudioPlugin` runs this once at startup on
/// native; the web defers to [`lazy_initialize_audio_system`] because
/// browsers refuse audio before a user gesture.
#[cfg(not(target_arch = "wasm32"))]
pub fn initialize_audio_system(world: &mut World) {
    let _span = tracing::info_span!("audio_init").entered();
    let is_initialized = audio_engine_is_initialized(world.plugin_resource::<AudioEngine>());
    if !is_initialized
        && let Err(error) = audio_engine_initialize(world.plugin_resource_mut::<AudioEngine>())
    {
        tracing::error!("Failed to initialize audio: {}", error);
    }
}

/// Opens the audio device once a user gesture has arrived, the web's
/// autoplay rule. `AudioPlugin` runs this every frame on the web; it no-ops
/// until the first keyboard or mouse event and once the device exists.
#[cfg(target_arch = "wasm32")]
pub fn lazy_initialize_audio_system(world: &mut World) {
    if audio_engine_is_initialized(world.plugin_resource::<AudioEngine>()) {
        return;
    }
    let gestured = world
        .expect_resource::<crate::ecs::input::resources::Input>()
        .events
        .iter()
        .any(|event| {
            matches!(
                event,
                crate::ecs::input::events::AppEvent::Keyboard { .. }
                    | crate::ecs::input::events::AppEvent::Mouse { .. }
            )
        });
    if !gestured {
        return;
    }
    if let Err(error) = audio_engine_initialize(world.plugin_resource_mut::<AudioEngine>()) {
        tracing::error!("Failed to initialize audio (WASM): {}", error);
    }
}

pub fn build_audio_buses_system(world: &mut World) {
    if world.plugin_resource::<AudioEngine>().buses_built {
        return;
    }
    let audio = world.plugin_resource_mut::<AudioEngine>();
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

    let default_reverb = match manager.add_send_track(
        SendTrackBuilder::new().with_effect(ReverbBuilder::new().mix(Mix::WET).damping(0.5)),
    ) {
        Ok(handle) => handle,
        Err(error) => {
            tracing::error!("Failed to create default reverb send: {}", error);
            return;
        }
    };

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
    audio
        .reverb_sends
        .insert("default".to_string(), default_reverb);
    audio.buses_built = true;
    tracing::info!("Audio buses built");
}

fn bus_handle_mut(
    audio: &mut crate::plugins::audio::resources::AudioEngine,
    bus: AudioBus,
) -> Option<&mut TrackHandle> {
    match bus {
        AudioBus::Master => audio.buses.master.as_mut(),
        AudioBus::Music => audio.buses.music.as_mut(),
        AudioBus::Sfx => audio.buses.sfx.as_mut(),
        AudioBus::Ambient => audio.buses.ambient.as_mut(),
        AudioBus::Voice => audio.buses.voice.as_mut(),
        AudioBus::Ui => audio.buses.ui.as_mut(),
    }
}

pub fn set_audio_bus_volume(world: &mut World, bus: AudioBus, decibels: f32, fade_seconds: f32) {
    if let Some(handle) = bus_handle_mut(world.plugin_resource_mut::<AudioEngine>(), bus) {
        handle.set_volume(
            Decibels(decibels),
            Tween {
                duration: Duration::from_secs_f32(fade_seconds.max(0.0)),
                ..Default::default()
            },
        );
    }
}

pub fn set_voice_ducking(world: &mut World, ducking: f32, fade_seconds: f32) {
    if let Some(tweener) = world
        .plugin_resource_mut::<AudioEngine>()
        .voice_ducking
        .as_mut()
    {
        tweener.set(
            ducking.clamp(0.0, 1.0) as f64,
            Tween {
                duration: Duration::from_secs_f32(fade_seconds.max(0.0)),
                ..Default::default()
            },
        );
    }
}

pub fn update_audio_system(world: &mut World) {
    let source_count = world.ecs.worlds[CORE]
        .query_entities(crate::ecs::world::AUDIO_SOURCE)
        .count();
    let _span = tracing::info_span!("audio", sources = source_count).entered();

    if !audio_engine_is_initialized(world.plugin_resource::<AudioEngine>()) {
        return;
    }
    if !world.plugin_resource::<AudioEngine>().buses_built {
        return;
    }

    let listener_entity = world.ecs.worlds[CORE]
        .query_entities(crate::ecs::world::AUDIO_LISTENER)
        .next();

    let has_listener_entity = listener_entity.is_some();

    if has_listener_entity {
        if world.plugin_resource::<AudioEngine>().listener.is_none() {
            let position = if let Some(entity) = listener_entity
                && let Some(transform) =
                    world.get::<crate::ecs::transform::components::GlobalTransform>(entity)
            {
                KiraVec3 {
                    x: transform.0[(0, 3)],
                    y: transform.0[(1, 3)],
                    z: transform.0[(2, 3)],
                }
            } else {
                KiraVec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                }
            };

            let orientation = nalgebra_glm::Quat::identity();
            let kira_orientation = KiraQuat {
                v: mint::Vector3 {
                    x: orientation.coords.x,
                    y: orientation.coords.y,
                    z: orientation.coords.z,
                },
                s: orientation.coords.w,
            };

            let listener_result = world
                .plugin_resource_mut::<AudioEngine>()
                .manager
                .as_mut()
                .map(|manager| manager.add_listener(position, kira_orientation));
            if let Some(result) = listener_result {
                match result {
                    Ok(listener) => {
                        world.plugin_resource_mut::<AudioEngine>().listener = Some(listener);
                        tracing::info!("Created audio listener");
                    }
                    Err(error) => {
                        tracing::error!("Failed to create audio listener: {}", error);
                    }
                }
            }
        }

        let (position, kira_orientation) = if let Some(entity) = listener_entity
            && let Some(transform) =
                world.get::<crate::ecs::transform::components::GlobalTransform>(entity)
        {
            let pos = KiraVec3 {
                x: transform.0[(0, 3)],
                y: transform.0[(1, 3)],
                z: transform.0[(2, 3)],
            };
            let rotation_matrix = nalgebra_glm::mat4_to_mat3(&transform.0);
            let orientation = nalgebra_glm::mat3_to_quat(&rotation_matrix);
            let quat = KiraQuat {
                v: mint::Vector3 {
                    x: orientation.coords.x,
                    y: orientation.coords.y,
                    z: orientation.coords.z,
                },
                s: orientation.coords.w,
            };
            (Some(pos), Some(quat))
        } else {
            (None, None)
        };

        if let (Some(pos), Some(quat)) = (position, kira_orientation)
            && let Some(listener_handle) = &mut world.plugin_resource_mut::<AudioEngine>().listener
        {
            listener_handle.set_position(pos, Tween::default());
            listener_handle.set_orientation(quat, Tween::default());
        }
    }

    let audio_entities: Vec<_> = world.ecs.worlds[CORE]
        .query_entities(crate::ecs::world::AUDIO_SOURCE)
        .collect();

    struct PlayIntent {
        entity: Entity,
        sound_data: StaticSoundData,
        is_spatial: bool,
        bus: AudioBus,
        min_distance: f32,
        max_distance: f32,
        reverb_zones: Vec<(String, f32)>,
        position: KiraVec3,
    }

    let mut sounds_to_play: Vec<PlayIntent> = Vec::new();
    let mut sounds_to_stop = Vec::new();
    let mut spatial_tracks_to_update = Vec::new();

    for entity in &audio_entities {
        let Some(source) = world.get::<crate::plugins::audio::components::AudioSource>(*entity)
        else {
            continue;
        };

        let is_spatial = source.spatial;
        let has_handle = audio_engine_has_handle(world.plugin_resource::<AudioEngine>(), *entity);
        let is_playing = source.playing;
        let audio_ref = pick_active_clip(
            source,
            *entity,
            world
                .expect_resource::<crate::ecs::time::Time>()
                .uptime_milliseconds,
        );
        let bus = source.bus;
        let min_distance = source.min_distance;
        let max_distance = source.max_distance;
        let reverb_zones = source.reverb_zones.clone();
        let looping = source.looping;
        let volume = source.volume;

        let playback_rate = source.playback_rate;

        if is_playing
            && !has_handle
            && let Some(ref audio_ref) = audio_ref
            && let Some(sound_data) =
                audio_engine_get_sound(world.plugin_resource::<AudioEngine>(), audio_ref)
        {
            let mut sound_data = sound_data.clone();
            if looping {
                sound_data = sound_data.loop_region(..);
            }
            sound_data = sound_data.volume(Decibels::from(volume));
            if (playback_rate - 1.0).abs() > f64::EPSILON {
                sound_data = sound_data.playback_rate(playback_rate);
            }

            let position = if is_spatial {
                world
                    .get::<crate::ecs::transform::components::GlobalTransform>(*entity)
                    .map(|transform| KiraVec3 {
                        x: transform.0[(0, 3)],
                        y: transform.0[(1, 3)],
                        z: transform.0[(2, 3)],
                    })
                    .unwrap_or(KiraVec3 {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    })
            } else {
                KiraVec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                }
            };

            sounds_to_play.push(PlayIntent {
                entity: *entity,
                sound_data,
                is_spatial,
                bus,
                min_distance,
                max_distance,
                reverb_zones,
                position,
            });
        } else if !is_playing && has_handle {
            sounds_to_stop.push(*entity);
        }

        if is_spatial
            && world
                .plugin_resource::<AudioEngine>()
                .spatial_tracks
                .contains_key(entity)
            && let Some(transform) =
                world.get::<crate::ecs::transform::components::GlobalTransform>(*entity)
        {
            let position = KiraVec3 {
                x: transform.0[(0, 3)],
                y: transform.0[(1, 3)],
                z: transform.0[(2, 3)],
            };
            spatial_tracks_to_update.push((*entity, position));
        }
    }

    for intent in sounds_to_play {
        let audio = world.plugin_resource_mut::<AudioEngine>();

        if intent.is_spatial && audio.listener.is_none() {
            tracing::warn!("Spatial audio requested but no listener exists");
            continue;
        }

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
            continue;
        };

        if intent.is_spatial {
            let listener = audio.listener.as_ref().unwrap();
            let reverb_sends = &audio.reverb_sends;

            let distance_input = (intent.min_distance as f64, intent.max_distance as f64);
            let mut track_builder = SpatialTrackBuilder::new()
                .distances((intent.min_distance, intent.max_distance))
                .attenuation_function(Easing::OutPowf(2.0))
                .with_effect(
                    FilterBuilder::new().cutoff(Value::FromListenerDistance(Mapping {
                        input_range: distance_input,
                        output_range: (20000.0, 500.0),
                        easing: Easing::OutPowf(2.0),
                    })),
                );

            for (zone_name, send_decibels) in &intent.reverb_zones {
                if let Some(send) = reverb_sends.get(zone_name) {
                    track_builder = track_builder.with_send(
                        send,
                        Value::FromListenerDistance(Mapping {
                            input_range: distance_input,
                            output_range: (Decibels(send_decibels - 6.0), Decibels(*send_decibels)),
                            easing: Easing::OutPowf(2.0),
                        }),
                    );
                } else {
                    tracing::warn!("Unknown reverb zone '{}'", zone_name);
                }
            }

            let track_result = bus.add_spatial_sub_track(listener, intent.position, track_builder);
            match track_result {
                Ok(mut spatial_track) => match spatial_track.play(intent.sound_data) {
                    Ok(sound_handle) => {
                        audio.spatial_tracks.insert(intent.entity, spatial_track);
                        audio
                            .spatial_sound_handles
                            .insert(intent.entity, sound_handle);
                    }
                    Err(error) => {
                        tracing::error!(
                            "Failed to play sound on spatial track for entity {:?}: {}",
                            intent.entity,
                            error
                        );
                    }
                },
                Err(error) => {
                    tracing::error!(
                        "Failed to create spatial track for entity {:?}: {}",
                        intent.entity,
                        error
                    );
                }
            }
        } else {
            let play_result = bus.play(intent.sound_data);
            match play_result {
                Ok(handle) => {
                    audio.sound_handles.insert(intent.entity, handle);
                }
                Err(error) => {
                    tracing::error!(
                        "Failed to play sound for entity {:?}: {}",
                        intent.entity,
                        error
                    );
                    if let Some(source) = world
                        .get_mut::<crate::plugins::audio::components::AudioSource>(intent.entity)
                    {
                        source.playing = false;
                    }
                }
            }
        }
    }

    for (entity, position) in spatial_tracks_to_update {
        if let Some(spatial_track) = world
            .plugin_resource_mut::<AudioEngine>()
            .spatial_tracks
            .get_mut(&entity)
        {
            spatial_track.set_position(position, Tween::default());
        }
    }

    for entity in sounds_to_stop {
        audio_engine_stop_sound(world.plugin_resource_mut::<AudioEngine>(), entity);
    }

    for entity in &audio_entities {
        let Some(source) = world.get::<crate::plugins::audio::components::AudioSource>(*entity)
        else {
            continue;
        };
        if !audio_engine_has_handle(world.plugin_resource::<AudioEngine>(), *entity) {
            continue;
        }
        let wants_pause = source.paused && source.playing;
        let currently_paused = world
            .plugin_resource::<AudioEngine>()
            .paused_handles
            .contains(entity);
        if wants_pause && !currently_paused {
            audio_engine_pause_sound(world.plugin_resource_mut::<AudioEngine>(), *entity);
            world
                .plugin_resource_mut::<AudioEngine>()
                .paused_handles
                .insert(*entity);
        } else if !wants_pause && currently_paused {
            audio_engine_resume_sound(world.plugin_resource_mut::<AudioEngine>(), *entity);
            world
                .plugin_resource_mut::<AudioEngine>()
                .paused_handles
                .remove(entity);
        }
    }
}

fn pick_active_clip(
    source: &AudioSource,
    entity: Entity,
    uptime_milliseconds: u64,
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
    if pool.is_empty() {
        return None;
    }
    let mut seed = uptime_milliseconds.wrapping_mul(0x9E3779B97F4A7C15);
    seed ^= (entity.id as u64).wrapping_mul(0xBF58476D1CE4E5B9);
    seed = seed.wrapping_mul(0x94D049BB133111EB);
    let index = ((seed >> 32) as usize) % pool.len();
    Some(pool[index].clone())
}

pub fn load_sound_from_bytes(
    bytes: &'static [u8],
) -> Result<StaticSoundData, Box<dyn std::error::Error>> {
    let cursor = Cursor::new(bytes);
    let sound_data = StaticSoundData::from_cursor(cursor)?;
    Ok(sound_data)
}

pub fn load_sound_from_cursor<T: AsRef<[u8]> + Send + Sync + 'static>(
    data: T,
) -> Result<StaticSoundData, Box<dyn std::error::Error>> {
    let cursor = Cursor::new(data);
    let sound_data = StaticSoundData::from_cursor(cursor)?;
    Ok(sound_data)
}
