use crate::commands::audio::{load_clips, open_track, play};
use crate::data::{SOUND_BATTLE_BED, SOUND_NIGHT_BED, SOUND_ZONE_BED};
use crate::queries::night_amount;
use crate::resources::{Cues, Feel, Tracks};
use nightshade::prelude::*;
use nightshade_audio::contract::AudioSource;
use nightshade_audio::resources::AudioEngine;
use nightshade_render::data::config::RendererState;

pub fn load(mut engine: ResMut<AudioEngine>, mut tracks: ResMut<Tracks>, mut commands: Commands) {
    load_clips(&mut engine);
    tracks.zone = Some(open_track(&mut commands, SOUND_ZONE_BED, true));
    tracks.night = Some(open_track(&mut commands, SOUND_NIGHT_BED, false));
    tracks.battle = Some(open_track(&mut commands, SOUND_BATTLE_BED, false));
}

pub fn cues(mut cues: ResMut<Cues>, mut commands: Commands) {
    for cue in std::mem::take(&mut cues.pending) {
        play(&mut commands, cue.clip, cue.volume);
    }
}

pub fn music(
    renderer_state: Res<RendererState>,
    feel: Res<Feel>,
    mut tracks: ResMut<Tracks>,
    mut sources: Query<(&mut AudioSource,)>,
) {
    let fighting = feel.combat > 0.0;
    let dark = night_amount(renderer_state.day_night.hour) > 0.6;
    if tracks.battle_on == fighting && tracks.night_on == dark {
        return;
    }
    tracks.battle_on = fighting;
    tracks.night_on = dark;
    if let Some(battle) = tracks.battle {
        sources.for_entity(battle, |(source,)| source.playing = fighting);
    }
    if let Some(zone) = tracks.zone {
        sources.for_entity(zone, |(source,)| source.playing = !dark);
    }
    if let Some(night) = tracks.night {
        sources.for_entity(night, |(source,)| source.playing = dark);
    }
}
