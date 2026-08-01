use nightshade::prelude::*;
use nightshade_platform::contract::input::Input;
use nightshade_platform::prelude::KeyCode;
use nightshade_readout::Readout;
use nightshade_story::prelude::{Play, Story, begin};
use nightshade_ui_story::StoryCard;

pub fn take_the_keys(input: Res<Input>, story: Res<Story>, mut play: ResMut<Play>) {
    if input.keyboard.just_pressed(KeyCode::KeyR) {
        begin(&mut play, &story);
    }
}

pub fn show_the_card(play: Res<Play>, card: Res<StoryCard>, mut readout: ResMut<Readout>) {
    readout.notes.clear();
    let written = card.shown.iter().filter(|line| !line.is_empty()).count();
    readout.notes.push(format!("{written} lines on the card"));
    if let Some(last) = card.shown.iter().rev().find(|line| !line.is_empty()) {
        readout.notes.push(last.clone());
    }
    readout
        .notes
        .push(format!("{} links offered", card.offered.len()));
    if play.ended {
        readout.notes.push("the story has ended".to_string());
    }
}
