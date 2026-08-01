use crate::theme;
use nightshade::prelude::*;
use nightshade_platform::contract::input::Input;
use nightshade_platform::prelude::KeyCode;
use nightshade_readout::Readout;
use nightshade_topdown::{Heading, Walking};

pub fn take_the_orders(input: Res<Input>, walkers: Query<(&mut Walking,)>) {
    let share = if input.keyboard.is_key_pressed(KeyCode::Space) {
        theme::RUNNING
    } else {
        theme::STROLLING
    };
    walkers.for_each(|_entity, (walking,)| {
        walking.0 = share;
    });
}

pub fn show_the_heading(mut readout: ResMut<Readout>, walkers: Query<(&Heading, &Walking)>) {
    let mut note = "still".to_string();
    let mut pace = theme::STROLLING;
    walkers.for_each(|_entity, (heading, walking)| {
        pace = walking.0;
        if heading.0.norm() > 0.01 {
            note = format!("{:.2}, {:.2}", heading.0.x, heading.0.y);
        }
    });
    readout.notes.clear();
    readout.notes.push(note);
    readout.notes.push(format!("pace {pace:.2}"));
}
