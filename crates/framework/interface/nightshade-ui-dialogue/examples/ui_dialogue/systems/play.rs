use crate::theme;
use nightshade::ecs::system_param::{Res, ResMut};
use nightshade_dialogue::Conversation;
use nightshade_dialogue::commands::{close, open, reveal_all, step};
use nightshade_dialogue::queries::{fully_shown, shown, talking};
use nightshade_platform::contract::input::Input;
use nightshade_platform::prelude::KeyCode;
use nightshade_readout::Readout;

pub fn take_the_orders(input: Res<Input>, mut conversation: ResMut<Conversation>) {
    if input.keyboard.just_pressed(KeyCode::KeyR) {
        open(&mut conversation, theme::SPEAKER, &theme::LINES);
        return;
    }
    if input.keyboard.just_pressed(KeyCode::KeyF) {
        close(&mut conversation);
        return;
    }
    if !input.keyboard.just_pressed(KeyCode::Space) || !talking(&conversation) {
        return;
    }
    if fully_shown(&conversation) {
        step(&mut conversation);
    } else {
        reveal_all(&mut conversation);
    }
}

pub fn watch_the_card(mut readout: ResMut<Readout>, conversation: Res<Conversation>) {
    readout.notes.clear();
    if !talking(&conversation) {
        readout.notes.push("nobody is speaking".to_string());
        return;
    }
    let letters = shown(&conversation).chars().count();
    let whole = conversation.line.chars().count();
    readout.notes.push(conversation.speaker.clone());
    readout.notes.push(format!("{letters} of {whole} letters"));
    readout
        .notes
        .push(format!("{} lines waiting", conversation.waiting.len()));
}
