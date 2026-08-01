use crate::resources::AppResources;
use nightshade::prelude::*;
use nightshade_audio::commands::audio_engine_load_sound_from_bytes;
use nightshade_audio::contract::{AudioBus, AudioSource};
use nightshade_audio::queries::audio_engine_get_sound;
use nightshade_audio::resources::AudioEngine;
use nightshade_platform::prelude::{Input, MouseState, Time};

const ROOM: &str = "ambience";
const HUM: &str = "hum";
const TOWER: &str = "bootup";
const CLICK: &str = "click";
const BLOOP: &str = "bloop";
const CHIME: &str = "chime";
const WARNING: &str = "warning";

const ROOM_STARTING: f32 = 0.20;
const ROOM_SETTLED: f32 = 0.11;
const HUM_VOLUME: f32 = 0.16;
const OVERLAP: f32 = 1.2;
const CLICK_VOLUME: f32 = 0.26;
const BLOOP_VOLUME: f32 = 0.32;
const CHIME_VOLUME: f32 = 0.38;
const WARNING_VOLUME: f32 = 0.30;
const TOWER_VOLUME: f32 = 0.50;

pub fn start(
    mut resources: ResMut<AppResources>,
    time: Res<Time>,
    mut engine: ResMut<AudioEngine>,
    mut commands: Commands,
    mut sources: Query<&mut AudioSource>,
) {
    let resources = &mut *resources;
    if !crate::systems::intro::reached_for(resources) {
        return;
    }
    if !resources.session.ambience_started {
        resources.session.ambience_started = true;

        for (name, bytes) in [
            (ROOM, include_bytes!("../../assets/ambience.mp3").as_slice()),
            (HUM, include_bytes!("../../assets/hum.mp3").as_slice()),
            (TOWER, include_bytes!("../../assets/bootup.mp3").as_slice()),
            (CLICK, include_bytes!("../../assets/click.mp3").as_slice()),
            (BLOOP, include_bytes!("../../assets/bloop.mp3").as_slice()),
            (CHIME, include_bytes!("../../assets/chime.mp3").as_slice()),
            (
                WARNING,
                include_bytes!("../../assets/warning.mp3").as_slice(),
            ),
        ] {
            audio_engine_load_sound_from_bytes(&mut engine, name, bytes.to_vec());
        }

        resources.session.hum_seconds = audio_engine_get_sound(&engine, HUM)
            .map(|sound| sound.duration().as_secs_f32())
            .unwrap_or(30.0);

        let room = commands.spawn((AudioSource {
            audio_ref: Some(ROOM.to_string()),
            volume: ROOM_STARTING,
            looping: true,
            playing: true,
            bus: AudioBus::Ambient,
            ..Default::default()
        },));
        resources.session.room_sound = Some(room);

        start_hum(resources, &mut commands);
    }

    run_hum(resources, &mut commands, time.delta_time);
    settle(resources, &mut sources);
}

const HUM_WEEK: f32 = 0.055;

fn start_hum(resources: &mut AppResources, commands: &mut Commands) {
    let nights = resources.script.nights.len().max(2);
    let through = resources.session.night as f32 / (nights - 1) as f32;
    let hum = commands.spawn((AudioSource {
        audio_ref: Some(HUM.to_string()),
        volume: HUM_VOLUME + HUM_WEEK * through.clamp(0.0, 1.0),
        looping: false,
        playing: true,
        bus: AudioBus::Ambient,
        ..Default::default()
    },));
    resources.session.hum_retiring = resources.session.hum_sound.replace(hum);
    resources.session.hum_due = (resources.session.hum_seconds - OVERLAP).max(1.0);
    resources.session.hum_elapsed = 0.0;
}

fn run_hum(resources: &mut AppResources, commands: &mut Commands, delta: f32) {
    resources.session.hum_elapsed += delta;

    if let Some(retiring) = resources.session.hum_retiring
        && resources.session.hum_elapsed >= OVERLAP
    {
        commands.despawn(retiring);
        resources.session.hum_retiring = None;
    }

    if resources.session.hum_elapsed >= resources.session.hum_due {
        start_hum(resources, commands);
    }
}

pub fn go_quiet(resources: &mut AppResources, commands: &mut Commands) {
    for source in [
        resources.session.hum_sound.take(),
        resources.session.hum_retiring.take(),
        resources.session.room_sound.take(),
    ]
    .into_iter()
    .flatten()
    {
        commands.despawn(source);
    }
    resources.session.hum_due = f32::MAX;
}

fn settle(resources: &mut AppResources, sources: &mut Query<&mut AudioSource>) {
    if resources.session.room_settled || crate::systems::intro::starting(resources) {
        return;
    }
    resources.session.room_settled = true;
    if let Some(room) = resources.session.room_sound {
        sources.for_entity(room, |source| source.volume = ROOM_SETTLED);
    }
}

pub fn click(resources: Res<AppResources>, input: Res<Input>, mut commands: Commands) {
    if !resources.session.ambience_started {
        return;
    }
    if !input.mouse.state.contains(MouseState::LEFT_JUST_PRESSED) {
        return;
    }
    commands.spawn((AudioSource {
        audio_ref: Some(CLICK.to_string()),
        volume: CLICK_VOLUME,
        looping: false,
        playing: true,
        bus: AudioBus::Ui,
        ..Default::default()
    },));
}

pub fn cues(
    mut resources: ResMut<AppResources>,
    mut commands: Commands,
    gathered: Res<crate::systems::desk::Gathered>,
) {
    let resources = &mut *resources;
    if !resources.session.ambience_started || resources.session.cues.is_empty() {
        resources.session.cues.clear();
        return;
    }
    let asked: Vec<crate::resources::Cue> = std::mem::take(&mut resources.session.cues);
    let messenger = crate::systems::desk::showing(
        &gathered.windows,
        crate::systems::setup::program_index("iam"),
    ) && resources.desk.screen == crate::data::machine::Screen::Desktop;
    let mut played = [false; 4];
    for cue in asked {
        let (slot, clip, volume) = match cue {
            crate::resources::Cue::Message => (0, BLOOP, BLOOP_VOLUME),
            crate::resources::Cue::Invite => (1, CHIME, CHIME_VOLUME),
            crate::resources::Cue::Refused => (2, WARNING, WARNING_VOLUME),
            crate::resources::Cue::Boot => (3, TOWER, TOWER_VOLUME),
        };
        let from_the_messenger = matches!(
            cue,
            crate::resources::Cue::Message | crate::resources::Cue::Invite
        );
        if played[slot] || (from_the_messenger && !messenger) {
            continue;
        }
        played[slot] = true;
        commands.spawn((AudioSource {
            audio_ref: Some(clip.to_string()),
            volume,
            looping: false,
            playing: true,
            bus: AudioBus::Ui,
            ..Default::default()
        },));
    }
}
