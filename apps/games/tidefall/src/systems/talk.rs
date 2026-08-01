use crate::data::{Ask, MARKER_SPEECH, RELIC_HUM, RELICS, SPEECHES, TALK_REACH};
use crate::queries::{
    Bodies, Relics, Speakers, collect_bodies, collect_relics, collect_speakers, nearest_body,
};
use crate::resources::{Mode, Talk, TidefallResources};
use nightshade::prelude::*;
use nightshade_input::Actions;
use nightshade_scene::prelude::*;

pub fn speak(
    actions: Res<Actions<Ask>>,
    mut resources: ResMut<TidefallResources>,
    mut commands: Commands,
    bodies: Bodies<'_, '_>,
    speakers: Speakers<'_, '_>,
    relics: Relics<'_, '_>,
    mut walkers: Query<(&LocalTransform,)>,
) {
    let Some(walker) = resources.walker else {
        return;
    };
    if resources.mode != Mode::Afoot {
        resources.talk = None;
        resources.readout.near_line = String::new();
        resources.readout.talk_shown = false;
        return;
    }
    let mut position = Vec3::zeros();
    walkers.for_entity(walker, |(transform,)| {
        position = transform.translation;
    });
    let views = collect_bodies(bodies);
    let spots = collect_speakers(speakers, &views);
    let shards = collect_relics(relics, &views);
    let here = nearest_body(position, &views).map(|index| views[index].entity);

    let speaker = spots
        .iter()
        .map(|spot| (spot, (spot.center - position).magnitude()))
        .filter(|(_, distance)| *distance <= TALK_REACH)
        .min_by(|(_, first), (_, second)| first.total_cmp(second))
        .map(|(spot, _)| spot);
    let shard = shards
        .iter()
        .filter(|shard| Some(shard.body) == here)
        .map(|shard| (shard, (shard.center - position).magnitude()))
        .min_by(|(_, first), (_, second)| first.total_cmp(second));

    let pressed = actions.just_pressed(Ask::Interact);
    let resources = &mut *resources;
    if pressed {
        if let Some(talk) = &mut resources.talk {
            talk.at += 1;
            if talk.at >= talk.lines.len() {
                resources.talk = None;
            }
        } else if let Some((shard, distance)) = shard
            && distance <= TALK_REACH
        {
            resources.found.insert(shard.index);
            commands.despawn_recursive(shard.entity);
            resources.talk = Some(Talk {
                name: format!("SHARD {}", shard.index + 1),
                lines: vec![RELICS[shard.index].text.to_string()],
                at: 0,
            });
        } else if let Some(spot) = speaker {
            let speech = &SPEECHES[spot.index];
            let lines = if resources.found.len() >= RELICS.len() && !speech.after.is_empty() {
                speech.after
            } else {
                speech.lines
            };
            resources.talk = Some(Talk {
                name: speech.name.to_string(),
                lines: lines.iter().map(|line| line.to_string()).collect(),
                at: 0,
            });
        }
    }

    resources.readout.near_line = if let Some(spot) = speaker {
        let speech = &SPEECHES[spot.index];
        if spot.index >= MARKER_SPEECH {
            "a waystone".to_string()
        } else {
            speech.name.to_string()
        }
    } else if let Some((_, distance)) = shard
        && distance <= TALK_REACH
    {
        "a shard".to_string()
    } else if let Some((_, distance)) = shard
        && distance <= RELIC_HUM
    {
        format!("a hum, {distance:.0} away")
    } else {
        String::new()
    };
    let readout = &mut resources.readout;
    match &resources.talk {
        Some(talk) => {
            readout.talk_shown = true;
            readout.talk_name = talk.name.clone();
            readout.talk_text = talk.lines[talk.at.min(talk.lines.len() - 1)].clone();
        }
        None => {
            readout.talk_shown = false;
        }
    }
}
