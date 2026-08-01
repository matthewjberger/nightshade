use crate::resources::Words;
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

pub fn take_the_orders(input: Res<Input>, settings: Res<ToastSettings>, mut toast: ResMut<Toast>) {
    let asked = if input.keyboard.just_pressed(KeyCode::Digit1) {
        Some(0)
    } else if input.keyboard.just_pressed(KeyCode::Digit2) {
        Some(1)
    } else if input.keyboard.just_pressed(KeyCode::Digit3) {
        Some(2)
    } else if input.keyboard.just_pressed(KeyCode::Digit4) {
        Some(3)
    } else {
        None
    };
    if let Some(step) = asked {
        say(&mut toast, &settings, theme::LINES[step]);
        return;
    }
    if input.keyboard.just_pressed(KeyCode::Space) {
        hush(&mut toast);
    }
}

pub fn watch_the_toast(
    toast_settings: Res<ToastSettings>,
    toast: Res<Toast>,
    words: Res<Words>,
    mut readout: ResMut<Readout>,
    storage: &mut Storage,
) {
    let (share, line, up) = {
        let settings = &*toast_settings;
        (
            opacity(&toast, settings),
            toast.line.clone(),
            showing(&toast),
        )
    };

    if let Some(card) = words.card
        && let Some(transform) = storage.get_mut::<LocalTransform>(card)
    {
        transform.scale.y = 0.05 + share * 1.4;
        transform.translation.y = 2.0 + share * 0.6;
    }

    readout.notes.clear();
    readout
        .notes
        .push(if up { line } else { "quiet".to_string() });
    readout.notes.push(format!("opacity {share:.2}"));
}
