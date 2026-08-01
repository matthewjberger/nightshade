use crate::theme;
use nightshade::ecs::system_param::{Res, ResMut};
use nightshade_platform::contract::input::Input;
use nightshade_platform::prelude::KeyCode;
use nightshade_readout::Readout;
use nightshade_ui_banner::Banner;
use nightshade_ui_banner::commands::{say, show, tint};

pub fn take_the_orders(input: Res<Input>, mut banner: ResMut<Banner>) {
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
        say(&mut banner, theme::LINES[step].to_string());
    }
    if input.keyboard.just_pressed(KeyCode::KeyR) {
        let next = if banner.color == theme::URGENT {
            theme::CALM
        } else {
            theme::URGENT
        };
        tint(&mut banner, next);
    }
    if input.keyboard.just_pressed(KeyCode::Space) {
        let hidden = !banner.shown;
        show(&mut banner, hidden);
    }
}

pub fn watch_the_banner(mut readout: ResMut<Readout>, banner: Res<Banner>) {
    readout.notes.clear();
    readout.notes.push(if banner.line.is_empty() {
        "blank".to_string()
    } else {
        banner.line.clone()
    });
    readout
        .notes
        .push(if banner.shown { "up" } else { "down" }.to_string());
    readout.notes.push(
        if banner.color == theme::URGENT {
            "urgent"
        } else {
            "calm"
        }
        .to_string(),
    );
}
