use crate::commands::audio::cue;
use crate::commands::banner;
use crate::commands::chat::{npc_says, system_line};
use crate::commands::effects::burst;
use crate::commands::spells::ascension;
use crate::commands::{grant_experience, inventory_take, schedule_line};
use crate::components::{Cooldowns, Experience, Giver, Level};
use crate::contract::{Channel, ChatLine};
use crate::data::chatter::DISCOVERY_LINES;
use crate::data::{
    GIVERS, Item, Objective, PLACES, QUESTS, SOUND_ACCEPT, SOUND_DISCOVER, SOUND_DONE,
    SOUND_OBJECTIVE,
};
use crate::queries::{coin_text, flat_distance, inventory_count, place_at};
use crate::resources::{
    Banners, BurstKind, Bursts, Cues, Hero, Inventory, Journal, Places, QuestState, Slain,
};
use crate::theme;
use nightshade::prelude::*;
use nightshade_scene::prelude::LocalTransform;

const ACTIVE_LIMIT: usize = 5;
const GOLD: Vec4 = Vec4::new(1.0, 0.85, 0.30, 1.0);

fn speak(journal: &mut Journal, giver: usize, lines: &[&str]) {
    let data = &GIVERS[giver];
    if data.board {
        let mut delay = 0.4;
        for text in lines {
            schedule_line(
                &mut journal.chat,
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
        npc_says(&mut journal.chat, data.name, lines, 0.4);
    }
}

pub fn givers(
    mut journal: ResMut<Journal>,
    mut inventory: ResMut<Inventory>,
    mut cues: ResMut<Cues>,
    mut bursts: ResMut<Bursts>,
    mut banners: ResMut<Banners>,
    mut commands: Commands,
    givers: Query<(&Giver, &LocalTransform)>,
    mut players: Query<(&LocalTransform, &mut Level, &mut Experience), With<Cooldowns>>,
) {
    let journal = &mut *journal;
    let mut position = None;
    players.for_each_mut(|_entity, (transform, _, _)| position = Some(transform.translation));
    let Some(position) = position else {
        return;
    };

    let mut spots = Vec::new();
    givers.for_each(|_entity, (giver, transform)| spots.push((giver.0, transform.translation)));

    for (giver, spot) in spots {
        let near = flat_distance(position, spot) < theme::GIVER_REACH;
        let entered = near && !journal.quests.near_giver[giver];
        journal.quests.near_giver[giver] = near;
        if !entered {
            continue;
        }

        let mut delivered = Vec::new();
        for state in journal.quests.active.iter_mut() {
            if let Objective::Deliver { giver: target } = QUESTS[state.quest].objective
                && target == giver
                && !state.complete
            {
                state.complete = true;
            }
            if state.complete && crate::queries::turn_in_giver(state.quest) == giver {
                delivered.push(state.quest);
            }
        }
        for quest in delivered {
            let data = &QUESTS[quest];
            if let Objective::Gather { pickup, count } = data.objective
                && !inventory_take(&mut inventory, Item::Loot(pickup), count)
            {
                continue;
            }
            journal.quests.active.retain(|state| state.quest != quest);
            if !data.repeatable {
                journal.quests.finished[quest] = true;
            }
            inventory.coins += data.coins;
            let mut leveled = false;
            players.for_each_mut(|_entity, (_, level, experience)| {
                leveled = grant_experience(level, experience, data.experience);
            });
            speak(journal, giver, data.complete);
            system_line(
                &mut journal.chat,
                format!(
                    "{} completed. {} experience and {}.",
                    data.title,
                    data.experience,
                    coin_text(data.coins)
                ),
            );
            cue(&mut cues, SOUND_DONE, theme::SOUND_LOUD);
            burst(&mut bursts, BurstKind::QuestDone, spot);
            ascension(&mut commands, spot);
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
                    &mut journal.chat,
                    "The quest pushed you over a level.".to_string(),
                );
            }
        }

        for (index, data) in QUESTS.iter().enumerate() {
            if data.giver != giver
                || journal.quests.active.len() >= ACTIVE_LIMIT
                || journal.quests.finished[index]
                || journal
                    .quests
                    .active
                    .iter()
                    .any(|state| state.quest == index)
            {
                continue;
            }
            journal.quests.active.push(QuestState {
                quest: index,
                progress: 0,
                complete: false,
            });
            speak(journal, giver, data.accept);
            system_line(
                &mut journal.chat,
                format!("Quest accepted: {}.", data.title),
            );
            cue(&mut cues, SOUND_ACCEPT, theme::SOUND_PLAIN);
            banner(
                &mut banners,
                data.title.to_string(),
                data.story.to_string(),
                GOLD,
            );
        }
    }
}

pub fn progress(mut journal: ResMut<Journal>, mut cues: ResMut<Cues>, slain: Res<Slain>) {
    let journal = &mut *journal;
    for event in slain.events.iter().filter(|event| event.by_player) {
        let mut notes = Vec::new();
        let mut finished = false;
        for state in journal.quests.active.iter_mut() {
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
                    GIVERS[crate::queries::turn_in_giver(state.quest)].name
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
                if finished {
                    theme::SOUND_PLAIN
                } else {
                    theme::SOUND_QUIET
                },
            );
        }
        for note in notes {
            system_line(&mut journal.chat, note);
        }
    }
}

pub fn tally(inventory: Res<Inventory>, mut journal: ResMut<Journal>, mut cues: ResMut<Cues>) {
    let journal = &mut *journal;
    let mut notes = Vec::new();
    for state in journal.quests.active.iter_mut() {
        let Objective::Gather { pickup, count } = QUESTS[state.quest].objective else {
            continue;
        };
        let held = inventory_count(&inventory, Item::Loot(pickup)).min(count);
        if held == state.progress {
            continue;
        }
        state.progress = held;
        let complete = held >= count;
        if complete && !state.complete {
            notes.push((
                true,
                format!(
                    "{} complete. Return to {}.",
                    QUESTS[state.quest].title,
                    GIVERS[crate::queries::turn_in_giver(state.quest)].name
                ),
            ));
        } else if !complete && held > 0 {
            notes.push((
                false,
                format!(
                    "{} gathered: {}/{}",
                    crate::data::PICKUPS[pickup].name,
                    held,
                    count
                ),
            ));
        }
        state.complete = complete;
    }
    for (finished, note) in notes {
        cue(
            &mut cues,
            if finished {
                SOUND_DONE
            } else {
                SOUND_OBJECTIVE
            },
            theme::SOUND_QUIET,
        );
        system_line(&mut journal.chat, note);
    }
}

pub fn explore(
    places: Res<Places>,
    mut hero: ResMut<Hero>,
    mut journal: ResMut<Journal>,
    mut cues: ResMut<Cues>,
    mut banners: ResMut<Banners>,
    players: Query<(&LocalTransform,), With<Cooldowns>>,
) {
    let journal = &mut *journal;
    let mut position = None;
    players.for_each(|_entity, (transform,)| position = Some(transform.translation));
    let Some(position) = position else {
        return;
    };
    let here = place_at(&places, position);
    if here != journal.clock.last_place {
        journal.clock.last_place = here;
        if let Some(place) = here
            && !journal.clock.discovered[place]
        {
            journal.clock.discovered[place] = true;
            let line = DISCOVERY_LINES[hero.random.usize(0..DISCOVERY_LINES.len())];
            cue(&mut cues, SOUND_DISCOVER, theme::SOUND_PLAIN);
            banner(
                &mut banners,
                PLACES[place].name.to_string(),
                line.to_string(),
                GOLD,
            );
            system_line(
                &mut journal.chat,
                format!("Discovered: {}.", PLACES[place].name),
            );
        }
    }

    let mut notes = Vec::new();
    for state in journal.quests.active.iter_mut() {
        let Objective::Discover { place } = QUESTS[state.quest].objective else {
            continue;
        };
        if state.complete || !journal.clock.discovered[place] {
            continue;
        }
        state.progress = 1;
        state.complete = true;
        notes.push(format!(
            "{} complete. {} found. Return to {}.",
            QUESTS[state.quest].title,
            PLACES[place].name,
            GIVERS[crate::queries::turn_in_giver(state.quest)].name
        ));
    }
    if notes.is_empty() {
        return;
    }
    cue(&mut cues, SOUND_DONE, theme::SOUND_PLAIN);
    for note in notes {
        system_line(&mut journal.chat, note);
    }
}
