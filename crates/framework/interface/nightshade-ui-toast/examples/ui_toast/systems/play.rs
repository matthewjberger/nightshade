use crate::resources::Sign;
use crate::theme;
use nightshade::ecs::Storage;
use nightshade::ecs::system_param::{Res, ResMut};
use nightshade_platform::contract::input::Input;
use nightshade_platform::prelude::KeyCode;
use nightshade_readout::Readout;
use nightshade_scene::contract::LocalTransform;
use nightshade_toast::commands::{hush, say};
use nightshade_toast::queries::{opacity, showing};
use nightshade_toast::{Toast, ToastSettings};

const DIGITS: [KeyCode; 4] = [
    KeyCode::Digit1,
    KeyCode::Digit2,
    KeyCode::Digit3,
    KeyCode::Digit4,
];

pub fn take_the_orders(input: Res<Input>, settings: Res<ToastSettings>, mut toast: ResMut<Toast>) {
    for (index, key) in DIGITS.iter().enumerate() {
        if input.keyboard.just_pressed(*key) {
            say(&mut toast, &settings, theme::LINES[index]);
        }
    }
    if input.keyboard.just_pressed(KeyCode::Space) {
        hush(&mut toast);
    }
}

pub fn watch_the_card(
    settings: Res<ToastSettings>,
    toast: Res<Toast>,
    sign: Res<Sign>,
    mut readout: ResMut<Readout>,
    storage: &mut Storage,
) {
    let share = opacity(&toast, &settings);
    let up = showing(&toast);
    if let Some(post) = sign.0
        && let Some(transform) = storage.get_mut::<LocalTransform>(post)
    {
        let height = theme::SIGN_LOW + share * (theme::SIGN_HIGH - theme::SIGN_LOW);
        transform.scale.y = height;
        transform.translation.y = height * 0.5;
    }

    readout.notes.clear();
    readout.notes.push(if up {
        format!("{:.1} seconds of card left", toast.remaining)
    } else {
        "the card is down".to_string()
    });
}
