use crate::commands::audio::{load_the_sounds, open_the_track};
use crate::resources::{Screen, Tracks};
use crate::theme;
use nightshade::prelude::*;
use nightshade_audio::contract::AudioSource;
use nightshade_audio::resources::AudioEngine;
use nightshade_state::prelude::*;

pub fn open_the_tracks(
    mut engine: ResMut<AudioEngine>,
    mut tracks: ResMut<Tracks>,
    mut commands: Commands,
) {
    load_the_sounds(&mut engine);
    tracks.camp = Some(open_the_track(
        &mut commands,
        theme::SOUND_CAMP,
        theme::MUSIC_LEVEL,
    ));
    tracks.battle = Some(open_the_track(
        &mut commands,
        theme::SOUND_BATTLE,
        theme::MUSIC_LEVEL,
    ));
}

pub fn follow_the_screen(
    mut tracks: ResMut<Tracks>,
    state_now: Res<State<Screen>>,
    sources: Query<(&mut AudioSource,)>,
) {
    let screen = state_now.current;
    if tracks.playing == Some(screen) {
        return;
    }
    tracks.playing = Some(screen);
    let camp = tracks.camp;
    let battle = tracks.battle;
    sources.for_each(|entity, (source,)| {
        if Some(entity) == camp {
            source.playing = matches!(screen, Screen::Camp | Screen::Title | Screen::Prologue);
        }
        if Some(entity) == battle {
            source.playing = screen == Screen::Battle;
        }
    });
}
