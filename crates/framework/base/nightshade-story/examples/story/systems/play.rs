use crate::commands::tint;
use crate::resources::Board;
use crate::theme;
use nightshade::ecs::Storage;
use nightshade::prelude::*;
use nightshade_platform::contract::input::Input;
use nightshade_platform::prelude::KeyCode;
use nightshade_readout::Readout;
use nightshade_render::resources::material_registry::MaterialRegistry;
use nightshade_scene::contract::LocalTransform;
use nightshade_story::prelude::{Deed, Place, Play, Story, begin, links, obey, pack, value};

const DIGITS: [KeyCode; 6] = [
    KeyCode::Digit1,
    KeyCode::Digit2,
    KeyCode::Digit3,
    KeyCode::Digit4,
    KeyCode::Digit5,
    KeyCode::Digit6,
];

pub fn take_the_keys(input: Res<Input>, story: Res<Story>, mut play: ResMut<Play>) {
    if input.keyboard.just_pressed(KeyCode::KeyR) {
        begin(&mut play, &story);
        return;
    }
    if input.keyboard.just_pressed(KeyCode::KeyL) {
        obey(&mut play, &story, Deed::Look);
        return;
    }
    if input.keyboard.just_pressed(KeyCode::KeyI) {
        obey(&mut play, &story, Deed::Held);
        return;
    }
    let offered = links(&story, &play);
    for (slot, key) in DIGITS.iter().enumerate() {
        if input.keyboard.just_pressed(*key)
            && let Some(link) = offered.get(slot)
        {
            obey(&mut play, &story, link.deed);
            return;
        }
    }
}

pub fn draw_the_board(
    play: Res<Play>,
    board: Res<Board>,
    mut material_registry: ResMut<MaterialRegistry>,
    mut commands: Commands,
    storage: &mut Storage,
) {
    let materials = &mut *material_registry;
    for (index, room) in board.rooms.iter().enumerate() {
        let standing = index == play.at;
        tint(
            materials,
            &mut commands,
            *room,
            if standing {
                theme::HERE_TINT
            } else {
                theme::ROOM_TINT
            },
        );
        if let Some(transform) = storage.get_mut::<LocalTransform>(*room) {
            transform.translation.y =
                theme::ROOM_LIFT + if standing { theme::HERE_RISE } else { 0.0 };
        }
    }
    for (index, thing) in board.things.iter().enumerate() {
        let color = match play.places.get(index).copied().unwrap_or_default() {
            Place::Held => theme::HELD_TINT,
            Place::Room(room) if room == play.at => theme::STANDING_TINT,
            Place::Room(_) => theme::ROOM_TINT,
            Place::Gone => theme::GONE_TINT,
        };
        tint(materials, &mut commands, *thing, color);
    }
}

pub fn show_the_state(story: Res<Story>, play: Res<Play>, mut readout: ResMut<Readout>) {
    readout.notes.clear();
    if let Some(room) = story.rooms.get(play.at) {
        readout.notes.push(room.title.clone());
    }
    if let Some(last) = play.told.last() {
        readout.notes.push(last.clone());
    }
    readout.notes.push(pack(&story, &play));
    readout
        .notes
        .push(format!("opened {}", value(&play, "opened")));
    let offered = links(&story, &play);
    for (slot, link) in offered.iter().take(DIGITS.len()).enumerate() {
        readout.notes.push(format!("{} {}", slot + 1, link.words));
    }
    if play.ended {
        readout.notes.push("the story has ended".to_string());
    }
}
