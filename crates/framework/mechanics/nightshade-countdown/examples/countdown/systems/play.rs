use crate::resources::Rounds;
use crate::theme;
use nightshade::ecs::Storage;
use nightshade::ecs::system_param::{EventReader, Res, ResMut};
use nightshade_countdown::commands::{spend, start, stop};
use nightshade_countdown::queries::{gone, seconds};
use nightshade_countdown::{RanOut, Remaining, Running};
use nightshade_platform::contract::input::Input;
use nightshade_platform::prelude::KeyCode;
use nightshade_readout::Readout;
use nightshade_scene::contract::LocalTransform;

pub fn take_the_orders(
    input: Res<Input>,
    mut remaining: ResMut<Remaining>,
    mut running: ResMut<Running>,
) {
    if input.keyboard.just_pressed(KeyCode::Space) {
        start(&mut remaining, &mut running, theme::SECONDS);
    }
    if input.keyboard.just_pressed(KeyCode::KeyQ) {
        stop(&mut running);
    }
    if input.keyboard.just_pressed(KeyCode::KeyF) {
        spend(&mut remaining, theme::PENALTY);
    }
}

pub fn count_the_rounds(ran_out: EventReader<RanOut>, mut rounds: ResMut<Rounds>) {
    rounds.rounds += ran_out.iter().count() as u32;
}

pub fn show_the_clock(
    remaining: Res<Remaining>,
    running: Res<Running>,
    rounds: Res<Rounds>,
    mut readout: ResMut<Readout>,
    storage: &mut Storage,
) {
    let left = seconds(&remaining);
    let share = (left / theme::SECONDS).clamp(0.0, 1.0);
    if let Some(bar) = rounds.bar
        && let Some(transform) = storage.get_mut::<LocalTransform>(bar)
    {
        transform.scale.x = (theme::BAR_WIDTH * share).max(0.001);
        transform.translation.x = (theme::BAR_WIDTH * share - theme::BAR_WIDTH) * 0.5;
    }

    readout.notes.clear();
    readout.notes.push(format!("{left:.1} seconds left"));
    readout.notes.push(if running.0 {
        "running".to_string()
    } else if gone(&remaining) {
        "gone".to_string()
    } else {
        "stopped".to_string()
    });
    readout.notes.push(format!("{} rounds gone", rounds.rounds));
}
