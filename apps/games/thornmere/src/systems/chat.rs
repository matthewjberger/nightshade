use crate::commands::audio::cue;
use crate::commands::chat::{random_fill, start_thread};
use crate::commands::push_chat;
use crate::components::{Class, Job, Task, Voice};
use crate::contract::{Channel, ChatLine};
use crate::data::chatter::{BOSS_DOWN, IDLE_THREADS, SINGLES, WHISPERS};
use crate::data::{
    CHAT_ROWS, CHATTER_MAX_SECONDS, CHATTER_MIN_SECONDS, ELITE, LINE_GAP_SECONDS, PLACES,
    SOUND_QUIET, SOUND_WHISPER, WHISPER_MAX_SECONDS, WHISPER_MIN_SECONDS,
};
use crate::resources::{Cues, Slain, Speaker, ThornmereResources};
use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_random::prelude::*;
use nightshade_scene::prelude::Name;
use nightshade_ui::resources::RetainedUiInteraction;

pub fn roster(
    mut resources: ResMut<ThornmereResources>,
    people: Query<(&Name, &Voice, &Class, &Task)>,
) {
    let mut speakers = Vec::new();
    people.for_each(|_entity, (name, voice, class, task)| {
        if task.0 != Job::Offline {
            speakers.push(Speaker {
                name: name.0.clone(),
                voice: voice.0,
                class: class.0,
            });
        }
    });
    resources.chat.roster = speakers;
}

pub fn react(mut resources: ResMut<ThornmereResources>, slain: Res<Slain>) {
    let resources = &mut *resources;
    for event in slain.events.iter().filter(|event| event.rank >= ELITE) {
        let mut fill = random_fill(
            &mut resources.random,
            &resources.chat.roster,
            &resources.player_name,
        );
        fill.monster = event.prey.clone();
        fill.killer = event.killer_name.clone();
        if let Some(place) = event.place {
            fill.place = PLACES[place].name.to_string();
        }
        let thread = &BOSS_DOWN[resources.random.usize(0..BOSS_DOWN.len())];
        start_thread(
            &mut resources.chat,
            &mut resources.random,
            thread,
            &fill,
            0.2,
        );
    }
}

pub fn conversations(time: Res<Time>, mut resources: ResMut<ThornmereResources>) {
    let resources = &mut *resources;
    let delta_time = time.delta_time;
    resources.chat.chatter_timer -= delta_time;
    resources.chat.whisper_timer -= delta_time;
    if resources.chat.roster.is_empty() {
        return;
    }

    if resources.chat.chatter_timer <= 0.0 {
        resources.chat.chatter_timer = between(
            &mut resources.random,
            CHATTER_MIN_SECONDS,
            CHATTER_MAX_SECONDS,
        );
        let fill = random_fill(
            &mut resources.random,
            &resources.chat.roster,
            &resources.player_name,
        );
        let thread = if resources.random.f32() < 0.6 {
            &IDLE_THREADS[resources.random.usize(0..IDLE_THREADS.len())]
        } else {
            &SINGLES[resources.random.usize(0..SINGLES.len())]
        };
        start_thread(
            &mut resources.chat,
            &mut resources.random,
            thread,
            &fill,
            0.0,
        );
    }

    if resources.chat.whisper_timer <= 0.0 {
        resources.chat.whisper_timer = between(
            &mut resources.random,
            WHISPER_MIN_SECONDS,
            WHISPER_MAX_SECONDS,
        );
        let fill = random_fill(
            &mut resources.random,
            &resources.chat.roster,
            &resources.player_name,
        );
        let thread = &WHISPERS[resources.random.usize(0..WHISPERS.len())];
        start_thread(
            &mut resources.chat,
            &mut resources.random,
            thread,
            &fill,
            0.0,
        );
    }
}

fn formatted(line: &ChatLine) -> String {
    let speaker = line.speaker.as_deref().unwrap_or("");
    match line.channel {
        Channel::General => format!("[General] {speaker}: {}", line.text),
        Channel::Trade => format!("[Trade] {speaker}: {}", line.text),
        Channel::Lfg => format!("[LookingForGroup] {speaker}: {}", line.text),
        Channel::Whisper => format!("{speaker} whispers: {}", line.text),
        Channel::Say => {
            if speaker.is_empty() {
                line.text.clone()
            } else {
                format!("{speaker} says: {}", line.text)
            }
        }
        Channel::System | Channel::Combat => line.text.clone(),
    }
}

pub fn deliver(time: Res<Time>, mut resources: ResMut<ThornmereResources>, mut cues: ResMut<Cues>) {
    let delta_time = time.delta_time;
    let chat = &mut resources.chat;
    chat.gap -= delta_time;
    for scheduled in chat.scheduled.iter_mut() {
        scheduled.delay -= delta_time;
    }
    let mut due = Vec::new();
    if chat.gap <= 0.0
        && let Some(index) = chat
            .scheduled
            .iter()
            .position(|scheduled| scheduled.delay <= 0.0)
    {
        due.push(chat.scheduled.remove(index).line);
        chat.gap = LINE_GAP_SECONDS;
    }
    for line in due {
        if line.channel == Channel::Whisper {
            cue(&mut cues, SOUND_WHISPER, SOUND_QUIET);
        }
        let text = formatted(&line);
        push_chat(chat, line.channel, text);
    }
}

pub fn collect(mut resources: ResMut<ThornmereResources>, events: EventReader<ChatLine>) {
    for event in &events {
        let text = formatted(event);
        push_chat(&mut resources.chat, event.channel, text);
    }
}

pub fn scroll(
    input: Res<Input>,
    interaction: Res<RetainedUiInteraction>,
    hierarchy: Res<nightshade::ecs::HierarchyIndex>,
    mut resources: ResMut<ThornmereResources>,
) {
    let wheel = input.mouse.wheel_delta.y;
    if wheel == 0.0 {
        return;
    }
    let panel = resources.hud.chat_panel;
    if panel == Entity::default() {
        return;
    }
    let mut hovered = interaction.hovered_entity;
    let mut over = false;
    for _ in 0..8 {
        match hovered {
            Some(entity) if entity == panel => {
                over = true;
                break;
            }
            Some(entity) => hovered = hierarchy.parent_of.get(&entity).copied(),
            None => break,
        }
    }
    if !over {
        return;
    }
    let chat = &mut resources.chat;
    let most = chat.lines.len().saturating_sub(CHAT_ROWS) as i64;
    let step: i64 = if wheel > 0.0 { 1 } else { -1 };
    chat.scroll = (chat.scroll as i64 + step).clamp(0, most) as usize;
}
