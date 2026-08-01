use crate::commands::random::stylize;
use crate::commands::{push_chat, schedule_line};
use crate::contract::{Channel, ChatLine};
use crate::data::chatter::{ITEMS, Thread};
use crate::data::{CLASSES, GUILDS, PLACES, QUESTS, REPLY_MAX_SECONDS, REPLY_MIN_SECONDS, SPAWNS};
use crate::queries::chat::fill_text;
use crate::resources::{ChatLog, Fill, Speaker};
use nightshade_random::prelude::*;

pub fn random_fill(random: &mut Random, roster: &[Speaker], player: &str) -> Fill {
    let mut speakers = Vec::new();
    if !roster.is_empty() {
        let mut picks: Vec<usize> = (0..roster.len()).collect();
        random.shuffle(&mut picks);
        for index in picks.into_iter().take(3) {
            speakers.push(roster[index].clone());
        }
    }
    while speakers.len() < 3 {
        speakers.push(Speaker {
            name: "Someone".to_string(),
            voice: 1,
            class: random.usize(0..CLASSES.len()),
        });
    }
    let monster = SPAWNS[random.usize(0..SPAWNS.len())].name.to_string();
    let place = PLACES[random.usize(1..PLACES.len())].name.to_string();
    let item = ITEMS[random.usize(0..ITEMS.len())].to_string();
    let price = format!("{}g", random.u32(1..40));
    let quest = QUESTS[random.usize(0..QUESTS.len())].title.to_string();
    let guild = GUILDS[random.usize(0..GUILDS.len())].to_string();
    let level = random.u32(3..21).to_string();
    let killer = speakers[0].name.clone();
    Fill {
        speakers,
        monster,
        place,
        item,
        price,
        quest,
        guild,
        player: player.to_string(),
        level,
        killer,
    }
}

pub fn start_thread(
    chat: &mut ChatLog,
    random: &mut Random,
    thread: &Thread,
    fill: &Fill,
    first_delay: f32,
) {
    let mut delay = first_delay;
    for (index, template) in thread.lines.iter().enumerate() {
        let speaker_slot = if thread.channel == Channel::Whisper {
            0
        } else {
            index % fill.speakers.len()
        };
        let speaker = &fill.speakers[speaker_slot];
        let line = if let Some(system_text) = template.strip_prefix('!') {
            ChatLine {
                channel: Channel::System,
                speaker: None,
                text: fill_text(system_text, fill),
            }
        } else {
            ChatLine {
                channel: thread.channel,
                speaker: Some(speaker.name.clone()),
                text: stylize(&fill_text(template, fill), speaker.voice, random),
            }
        };
        schedule_line(chat, delay, line);
        delay += between(random, REPLY_MIN_SECONDS, REPLY_MAX_SECONDS);
    }
}

pub fn npc_says(chat: &mut ChatLog, speaker: &str, lines: &[&str], first_delay: f32) {
    let mut delay = first_delay;
    for text in lines {
        schedule_line(
            chat,
            delay,
            ChatLine {
                channel: Channel::Say,
                speaker: Some(speaker.to_string()),
                text: text.to_string(),
            },
        );
        delay += 2.8;
    }
}

pub fn system_line(chat: &mut ChatLog, text: String) {
    push_chat(chat, Channel::System, text);
}
