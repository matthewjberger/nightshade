use crate::commands::audio::cue;
use crate::commands::banner;
use crate::commands::chat::{npc_says, system_line};
use crate::commands::effects::burst;
use crate::commands::spells::ascension;
use crate::commands::{grant_experience, schedule_line};
use crate::components::{Cooldowns, Experience, Giver, Level};
use crate::contract::{Channel, ChatLine};
use crate::data::{
    GIVER_REACH, GIVERS, Objective, PLACES, QUESTS, SOUND_ACCEPT, SOUND_DONE, SOUND_LOUD,
    SOUND_OBJECTIVE, SOUND_PLAIN, SOUND_QUIET,
};
use crate::queries::{coin_text, flat_distance};
use crate::resources::{Banners, BurstKind, Bursts, Cues, QuestState, Slain, ThornmereResources};
use nightshade::prelude::*;
use nightshade_scene::prelude::LocalTransform;

const ACTIVE_LIMIT: usize = 5;
const GOLD: Vec4 = Vec4::new(1.0, 0.85, 0.30, 1.0);

pub fn turn_in_giver(quest: usize) -> usize {
    match QUESTS[quest].objective {
        Objective::Deliver { giver } => giver,
        _ => QUESTS[quest].giver,
    }
}

fn speak(resources: &mut ThornmereResources, giver: usize, lines: &[&str]) {
    let data = &GIVERS[giver];
    if data.board {
        let mut delay = 0.4;
        for text in lines {
            schedule_line(
                &mut resources.chat,
                delay,
                ChatLine {
                    channel: Channel::Say,
                    speaker: None,
                    text: format!("The board reads: {text}"),
                },
            );
            delay += 2.8;
        }
    } else {
        npc_says(&mut resources.chat, data.name, lines, 0.4);
    }
}

pub fn givers(
    mut resources: ResMut<ThornmereResources>,
    mut cues: ResMut<Cues>,
    mut bursts: ResMut<Bursts>,
    mut banners: ResMut<Banners>,
    mut commands: Commands,
    givers: Query<(&Giver, &LocalTransform)>,
    mut players: Query<(&LocalTransform, &mut Level, &mut Experience), With<Cooldowns>>,
) {
    let resources = &mut *resources;
    let mut position = None;
    players.for_each_mut(|_entity, (transform, _, _)| position = Some(transform.translation));
    let Some(position) = position else {
        return;
    };

    let mut spots = Vec::new();
    givers.for_each(|_entity, (giver, transform)| spots.push((giver.0, transform.translation)));

    for (giver, spot) in spots {
        let near = flat_distance(position, spot) < GIVER_REACH;
        let entered = near && !resources.quests.near_giver[giver];
        resources.quests.near_giver[giver] = near;
        if !entered {
            continue;
        }

        let mut delivered = Vec::new();
        for state in resources.quests.active.iter_mut() {
            if let Objective::Deliver { giver: target } = QUESTS[state.quest].objective
                && target == giver
                && !state.complete
            {
                state.complete = true;
            }
            if state.complete && turn_in_giver(state.quest) == giver {
                delivered.push(state.quest);
            }
        }
        for quest in delivered {
            resources.quests.active.retain(|state| state.quest != quest);
            let data = &QUESTS[quest];
            if !data.repeatable {
                resources.quests.finished[quest] = true;
            }
            resources.purse += data.coins;
            let mut leveled = false;
            players.for_each_mut(|_entity, (_, level, experience)| {
                leveled = grant_experience(level, experience, data.experience);
            });
            speak(resources, giver, data.complete);
            system_line(
                &mut resources.chat,
                format!(
                    "{} completed. {} experience and {}.",
                    data.title,
                    data.experience,
                    coin_text(data.coins)
                ),
            );
            cue(&mut cues, SOUND_DONE, SOUND_LOUD);
            burst(&mut bursts, BurstKind::QuestDone, spot);
            ascension(&mut commands, &resources.palette, spot);
            banner(
                &mut banners,
                "Quest Complete".to_string(),
                format!(
                    "{}   {} experience and {}",
                    data.title,
                    data.experience,
                    coin_text(data.coins)
                ),
                GOLD,
            );
            if leveled {
                system_line(
                    &mut resources.chat,
                    "The quest pushed you over a level.".to_string(),
                );
            }
        }

        for (index, data) in QUESTS.iter().enumerate() {
            if data.giver != giver
                || resources.quests.active.len() >= ACTIVE_LIMIT
                || resources.quests.finished[index]
                || resources
                    .quests
                    .active
                    .iter()
                    .any(|state| state.quest == index)
            {
                continue;
            }
            resources.quests.active.push(QuestState {
                quest: index,
                progress: 0,
                complete: false,
            });
            speak(resources, giver, data.accept);
            system_line(
                &mut resources.chat,
                format!("Quest accepted: {}.", data.title),
            );
            cue(&mut cues, SOUND_ACCEPT, SOUND_PLAIN);
            banner(
                &mut banners,
                data.title.to_string(),
                data.story.to_string(),
                GOLD,
            );
        }
    }
}

pub fn progress(
    mut resources: ResMut<ThornmereResources>,
    mut cues: ResMut<Cues>,
    slain: Res<Slain>,
) {
    let resources = &mut *resources;
    for event in slain.events.iter().filter(|event| event.by_player) {
        let mut notes = Vec::new();
        let mut finished = false;
        for state in resources.quests.active.iter_mut() {
            let Objective::Kill { prey, count } = QUESTS[state.quest].objective else {
                continue;
            };
            if state.complete || prey != event.prey {
                continue;
            }
            state.progress += 1;
            if state.progress >= count {
                state.complete = true;
                finished = true;
                notes.push(format!(
                    "{} complete. Return to {}.",
                    QUESTS[state.quest].title,
                    GIVERS[turn_in_giver(state.quest)].name
                ));
            } else {
                notes.push(format!("{} slain: {}/{}", prey, state.progress, count));
            }
        }
        if !notes.is_empty() {
            cue(
                &mut cues,
                if finished {
                    SOUND_DONE
                } else {
                    SOUND_OBJECTIVE
                },
                if finished { SOUND_PLAIN } else { SOUND_QUIET },
            );
        }
        for note in notes {
            system_line(&mut resources.chat, note);
        }
    }
}

pub fn explore(mut resources: ResMut<ThornmereResources>, mut cues: ResMut<Cues>) {
    let resources = &mut *resources;
    let mut notes = Vec::new();
    for state in resources.quests.active.iter_mut() {
        let Objective::Discover { place } = QUESTS[state.quest].objective else {
            continue;
        };
        if state.complete || !resources.clock.discovered[place] {
            continue;
        }
        state.progress = 1;
        state.complete = true;
        notes.push(format!(
            "{} complete. {} found. Return to {}.",
            QUESTS[state.quest].title,
            PLACES[place].name,
            GIVERS[turn_in_giver(state.quest)].name
        ));
    }
    if notes.is_empty() {
        return;
    }
    cue(&mut cues, SOUND_DONE, SOUND_PLAIN);
    for note in notes {
        system_line(&mut resources.chat, note);
    }
}
