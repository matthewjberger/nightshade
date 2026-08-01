use crate::data::chat::{
    Channel, GREETING_ASIDES, GREETING_REPLIES, GREETING_WORDS, KEYWORD_REPLIES, SECRET_PHRASE,
    SECRET_RENOWN, SECRET_REPLY,
};
use crate::queries::formatted;
use crate::resources::{Chat, ChatLine, Chatter, Roster, Scheduled};
use crate::theme;
use blocker_quests::commands::reward;
use blocker_quests::resources::Journal;
use blocker_screen::data::notice::Notice;
use nightshade::ecs::system_param::EventWriter;
use nightshade_random::Random;
use nightshade_random::commands::{chance, index, shuffle};

pub fn push_chat(chat: &mut Chat, channel: Channel, line: String) {
    chat.lines.push_back((channel, line));
    while chat.lines.len() > theme::CHAT_KEEP {
        chat.lines.pop_front();
    }
    chat.counts[channel.index()] += 1;
}

pub fn schedule_line(chatter: &mut Chatter, delay: f32, line: ChatLine) {
    chatter.scheduled.push(Scheduled { delay, line });
}

pub fn narrate(chat: &mut Chat, text: &str) {
    push_chat(chat, Channel::System, text.to_string());
}

pub fn npc_say(chat: &mut Chat, name: &str, text: &str) {
    let line = ChatLine {
        channel: Channel::Say,
        speaker: Some(name.to_string()),
        text: text.to_string(),
    };
    let rendered = formatted(&line);
    push_chat(chat, Channel::Say, rendered);
}

pub fn system_line(chat: &mut Chat, text: String) {
    push_chat(chat, Channel::System, text);
}

pub fn greet_the_speaker(
    chatter: &mut Chatter,
    roster: &Roster,
    random: &mut Random,
    player: &str,
) {
    if roster.0.is_empty() {
        return;
    }
    let mut names: Vec<String> = roster
        .0
        .iter()
        .map(|speaker| speaker.name.clone())
        .collect();
    shuffle(random, &mut names);
    let mut delay = 0.0;
    for (turn, name) in names.into_iter().take(theme::GREET_BACK_COUNT).enumerate() {
        let text = if turn == 0 || chance(random, 0.5) {
            GREETING_REPLIES[index(random, GREETING_REPLIES.len())].replace("{player}", player)
        } else {
            GREETING_ASIDES[index(random, GREETING_ASIDES.len())].to_string()
        };
        schedule_line(
            chatter,
            delay,
            ChatLine {
                channel: Channel::Say,
                speaker: Some(name),
                text,
            },
        );
        delay += theme::GREET_BACK_DELAY;
    }
}

pub fn respond_to_keywords(
    chat: &mut Chat,
    chatter: &mut Chatter,
    roster: &Roster,
    random: &mut Random,
    journal: &mut Journal,
    notices: &mut EventWriter<'_, Notice>,
    text: &str,
    player: &str,
) {
    let lower = text.to_lowercase();
    let greeted = lower
        .split(|character: char| !character.is_ascii_alphabetic())
        .any(|word| GREETING_WORDS.contains(&word));
    if greeted {
        greet_the_speaker(chatter, roster, random, player);
        return;
    }
    if !journal.found_secret && lower.contains(SECRET_PHRASE) {
        journal.found_secret = true;
        narrate(chat, SECRET_REPLY);
        reward(
            journal,
            notices,
            SECRET_RENOWN,
            0,
            0,
            "A secret is yours".to_string(),
        );
        return;
    }
    for &(needle, reply) in KEYWORD_REPLIES {
        if lower.contains(needle) {
            let speaker = roster
                .0
                .get(index(random, roster.0.len()))
                .map(|speaker| speaker.name.clone());
            match speaker {
                Some(name) => npc_say(chat, &name, reply),
                None => narrate(chat, reply),
            }
            return;
        }
    }
}
