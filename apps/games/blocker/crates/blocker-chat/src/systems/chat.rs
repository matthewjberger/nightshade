use crate::commands::push_chat;
use crate::data::chat::{Channel, SAY_LINES};
use crate::queries::formatted;
use crate::resources::{Chat, ChatLine, Chatter, Roster, Speaker};
use crate::theme;
use blocker_figure::components::{Eye, Home};
use blocker_figure::queries::read_the_hero;
use blocker_figure::resources::Hero;
use nightshade::prelude::*;
use nightshade_platform::prelude::Time;
use nightshade_random::Random;
use nightshade_random::commands::index;
use nightshade_scene::prelude::{LocalTransform, Name};

pub mod banter;
pub mod world;

use self::banter::{class_of, voice_of};

pub fn greet_the_player(
    time: Res<Time>,
    hero: Res<Hero>,
    mut chat: ResMut<Chat>,
    mut chatter: ResMut<Chatter>,
    mut random: ResMut<Random>,
    villagers: Query<(&Name, &LocalTransform), With<Home>>,
    mut bodies: Query<(&Eye,)>,
) {
    let Some(eye) = read_the_hero(&hero, &mut bodies, |(sight,)| sight.0) else {
        return;
    };
    chatter.greet_timer -= time.delta_time.min(0.05);
    if chatter.greet_timer > 0.0 {
        return;
    }
    let mut nearest: Option<(f32, String)> = None;
    villagers.for_each(|_entity, (name, transform)| {
        let across = transform.translation.x - eye.x;
        let along = transform.translation.z - eye.z;
        let distance = across * across + along * along;
        let closer = nearest.as_ref().is_none_or(|(best, _)| distance < *best);
        if distance < theme::GREET_RANGE * theme::GREET_RANGE && closer {
            nearest = Some((distance, name.0.clone()));
        }
    });
    if let Some((_, name)) = nearest {
        chatter.greet_timer = theme::GREET_GAP;
        let line = SAY_LINES[index(&mut random, SAY_LINES.len())];
        let rendered = formatted(&ChatLine {
            channel: Channel::Say,
            speaker: Some(name),
            text: line.to_string(),
        });
        push_chat(&mut chat, Channel::Say, rendered);
    }
}

pub fn gather_roster(mut roster: ResMut<Roster>, mut villagers: Query<(&Name,), With<Home>>) {
    let mut count = 0usize;
    villagers.for_each_mut(|_entity, _| count += 1);
    if count == roster.0.len() {
        return;
    }
    let mut speakers = Vec::with_capacity(count);
    villagers.for_each_mut(|_entity, (name,)| {
        speakers.push(Speaker {
            name: name.0.clone(),
            voice: voice_of(&name.0),
            class: class_of(&name.0),
        });
    });
    roster.0 = speakers;
}

pub fn deliver_the_lines(time: Res<Time>, mut chat: ResMut<Chat>, mut chatter: ResMut<Chatter>) {
    let delta_time = time.delta_time.min(0.05);
    chatter.gap -= delta_time;
    for scheduled in chatter.scheduled.iter_mut() {
        scheduled.delay -= delta_time;
    }
    if chatter.gap > 0.0 {
        return;
    }
    let Some(index) = chatter
        .scheduled
        .iter()
        .position(|scheduled| scheduled.delay <= 0.0)
    else {
        return;
    };
    let line = chatter.scheduled.remove(index).line;
    chatter.gap = theme::CHAT_GAP;
    let text = formatted(&line);
    push_chat(&mut chat, line.channel, text);
}
