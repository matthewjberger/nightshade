use crate::commands::schedule_line;
use crate::data::chat::{
    CLASSES, Channel, GUILDS, IDLE_THREADS, LOOT, MONSTERS, PLACES, QUEST_NAMES, SAY_LINES,
    SINGLES, Thread, WHISPERS,
};
use crate::resources::{ChatLine, Chatter, Fill, Roster, Speaker};
use crate::theme;
use blocker_player::resources::Profile;
use nightshade::prelude::*;
use nightshade_platform::prelude::Time;
use nightshade_random::Random;
use nightshade_random::commands::{between, chance, index, shuffle};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

fn fingerprint(name: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    name.hash(&mut hasher);
    hasher.finish()
}

pub(super) fn voice_of(name: &str) -> usize {
    (fingerprint(name) % 4) as usize
}

pub(super) fn class_of(name: &str) -> usize {
    (fingerprint(name) >> 32) as usize % CLASSES.len()
}

pub(super) fn stylize(text: &str, voice: usize, random: &mut Random) -> String {
    let mut line = text.to_string();
    match voice {
        0 => {
            line = line.to_lowercase();
            while line.ends_with('.') {
                line.pop();
            }
        }
        1 => {
            let mut characters = line.chars();
            if let Some(first) = characters.next() {
                line = first.to_uppercase().collect::<String>() + characters.as_str();
            }
            if !line.ends_with(['.', '!', '?']) {
                line.push('.');
            }
        }
        2 => {
            while line.ends_with('.') {
                line.pop();
            }
            line.push_str(if chance(random, 0.5) { "!" } else { "!!" });
        }
        _ => {
            line = line.to_lowercase();
        }
    }
    line
}

pub(super) fn fill_text(template: &str, fill: &Fill) -> String {
    let class = &CLASSES[fill.speakers[0].class];
    template
        .replace("{a}", &fill.speakers[0].name)
        .replace("{b}", &fill.speakers[1].name)
        .replace("{c}", &fill.speakers[2].name)
        .replace("{class}", class.name)
        .replace("{role}", class.role)
        .replace("{monster}", &fill.monster)
        .replace("{place}", &fill.place)
        .replace("{item}", &fill.item)
        .replace("{price}", &fill.price)
        .replace("{quest}", &fill.quest)
        .replace("{guild}", &fill.guild)
        .replace("{player}", &fill.player)
        .replace("{level}", &fill.level)
        .replace("{killer}", &fill.killer)
}

pub(super) fn random_fill(roster: &Roster, random: &mut Random, player: &str) -> Fill {
    let mut speakers = Vec::new();
    if !roster.0.is_empty() {
        let mut picks: Vec<usize> = (0..roster.0.len()).collect();
        shuffle(random, &mut picks);
        for pick in picks.into_iter().take(3) {
            speakers.push(roster.0[pick].clone());
        }
    }
    while speakers.len() < 3 {
        let class = index(random, CLASSES.len());
        speakers.push(Speaker {
            name: "Someone".to_string(),
            voice: 1,
            class,
        });
    }
    let monster = MONSTERS[index(random, MONSTERS.len())].to_string();
    let place = PLACES[index(random, PLACES.len())].to_string();
    let item = LOOT[index(random, LOOT.len())].to_string();
    let price = format!("{}g", random.u32(1..40));
    let quest = QUEST_NAMES[index(random, QUEST_NAMES.len())].to_string();
    let guild = GUILDS[index(random, GUILDS.len())].to_string();
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

pub(super) fn start_thread(
    chatter: &mut Chatter,
    random: &mut Random,
    thread: &Thread,
    fill: &Fill,
    first_delay: f32,
) {
    let mut delay = first_delay;
    for (turn, template) in thread.lines.iter().enumerate() {
        let speaker_slot = if thread.channel == Channel::Whisper {
            0
        } else {
            turn % fill.speakers.len()
        };
        let speaker = &fill.speakers[speaker_slot];
        let line = if let Some(system_text) = template.strip_prefix('!') {
            ChatLine {
                channel: Channel::System,
                speaker: None,
                text: fill_text(system_text, fill),
            }
        } else {
            let voice = speaker.voice;
            let name = speaker.name.clone();
            let text = stylize(&fill_text(template, fill), voice, random);
            ChatLine {
                channel: thread.channel,
                speaker: Some(name),
                text,
            }
        };
        schedule_line(chatter, delay, line);
        delay += between(random, theme::REPLY_MIN, theme::REPLY_MAX);
    }
}

pub fn conversations(
    time: Res<Time>,
    profile: Res<Profile>,
    roster: Res<Roster>,
    mut chatter: ResMut<Chatter>,
    mut random: ResMut<Random>,
) {
    let delta_time = time.delta_time.min(0.05);
    chatter.chatter_timer -= delta_time;
    chatter.whisper_timer -= delta_time;
    if roster.0.is_empty() {
        return;
    }
    if chatter.chatter_timer <= 0.0 {
        chatter.chatter_timer = between(&mut random, theme::CHATTER_MIN, theme::CHATTER_MAX);
        if chance(&mut random, 0.25) {
            let name = roster.0[index(&mut random, roster.0.len())].name.clone();
            let line = SAY_LINES[index(&mut random, SAY_LINES.len())];
            schedule_line(
                &mut chatter,
                0.0,
                ChatLine {
                    channel: Channel::Say,
                    speaker: Some(name),
                    text: line.to_string(),
                },
            );
        } else {
            let fill = random_fill(&roster, &mut random, &profile.name);
            let thread = if chance(&mut random, 0.6) {
                &IDLE_THREADS[index(&mut random, IDLE_THREADS.len())]
            } else {
                &SINGLES[index(&mut random, SINGLES.len())]
            };
            start_thread(&mut chatter, &mut random, thread, &fill, 0.0);
        }
    }
    if chatter.whisper_timer <= 0.0 {
        chatter.whisper_timer = between(&mut random, theme::WHISPER_MIN, theme::WHISPER_MAX);
        let fill = random_fill(&roster, &mut random, &profile.name);
        let thread = &WHISPERS[index(&mut random, WHISPERS.len())];
        start_thread(&mut chatter, &mut random, thread, &fill, 0.0);
    }
}
