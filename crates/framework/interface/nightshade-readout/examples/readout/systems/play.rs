use crate::resources::Stack;
use crate::theme;
use nightshade::ecs::Storage;
use nightshade::ecs::system_param::{Res, ResMut};
use nightshade_platform::contract::input::Input;
use nightshade_platform::prelude::KeyCode;
use nightshade_readout::Readout;
use nightshade_readout::commands::{clear_notes, say};
use nightshade_scene::contract::LocalTransform;

const DIGITS: [KeyCode; theme::POSTS] = [
    KeyCode::Digit1,
    KeyCode::Digit2,
    KeyCode::Digit3,
    KeyCode::Digit4,
    KeyCode::Digit5,
];

pub fn take_the_orders(input: Res<Input>, mut readout: ResMut<Readout>) {
    for (index, key) in DIGITS.iter().enumerate() {
        if input.keyboard.just_pressed(*key) && readout.notes.len() < theme::POSTS {
            say(&mut readout, theme::LINES[index]);
        }
    }
    if input.keyboard.just_pressed(KeyCode::KeyZ) {
        clear_notes(&mut readout);
    }
}

pub fn show_the_stack(stack: Res<Stack>, readout: Res<Readout>, storage: &mut Storage) {
    let said = readout.notes.len();
    for (index, post) in stack.posts.iter().enumerate() {
        let height = if index < said {
            theme::POST_HIGH
        } else {
            theme::POST_LOW
        };
        if let Some(transform) = storage.get_mut::<LocalTransform>(*post) {
            transform.scale.y = height;
            transform.translation.y = height * 0.5;
        }
    }
}
