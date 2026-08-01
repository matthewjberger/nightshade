use crate::resources::Row;
use crate::theme;
use nightshade::prelude::*;
use nightshade_hover::queries::lift;
use nightshade_hover::{Bob, HoverSettings, Perch, Phase, Spin};
use nightshade_platform::contract::input::Input;
use nightshade_platform::contract::time::Time;
use nightshade_platform::prelude::KeyCode;
use nightshade_readout::Readout;
use nightshade_scene::contract::LocalTransform;

pub fn take_the_orders(
    input: Res<Input>,
    time: Res<Time>,
    mut row: ResMut<Row>,
    mut settings: ResMut<HoverSettings>,
    mut coins: Query<(&mut Perch, &mut Bob, &mut Spin)>,
) {
    if row.coins.is_empty() {
        return;
    }
    let last = row.coins.len() - 1;
    if input.keyboard.just_pressed(KeyCode::ArrowLeft) {
        row.chosen = row.chosen.saturating_sub(1);
    }
    if input.keyboard.just_pressed(KeyCode::ArrowRight) {
        row.chosen = (row.chosen + 1).min(last);
    }
    if input.keyboard.just_pressed(KeyCode::KeyQ) {
        settings.rate = (settings.rate - theme::RATE_STEP).max(0.0);
    }
    if input.keyboard.just_pressed(KeyCode::KeyE) {
        settings.rate = (settings.rate + theme::RATE_STEP).min(theme::RATE_HIGH);
    }

    let chosen = row.coins[row.chosen];
    let mut climb = 0.0;
    if input.keyboard.is_key_pressed(KeyCode::ArrowUp) {
        climb += theme::RISE * time.delta_time;
    }
    if input.keyboard.is_key_pressed(KeyCode::ArrowDown) {
        climb -= theme::RISE * time.delta_time;
    }
    let thinner = input.keyboard.just_pressed(KeyCode::KeyZ);
    let wider = input.keyboard.just_pressed(KeyCode::KeyX);
    let turning = input.keyboard.just_pressed(KeyCode::Space);
    coins.for_entity(chosen, |(perch, bob, spin)| {
        if climb != 0.0 {
            perch.0.y = (perch.0.y + climb).clamp(theme::PERCH_LOW, theme::PERCH_HIGH);
        }
        if thinner {
            bob.0 = (bob.0 - theme::BOB_STEP).max(0.0);
        }
        if wider {
            bob.0 = (bob.0 + theme::BOB_STEP).min(theme::BOB_MOST);
        }
        if turning {
            spin.0 = if spin.0 != 0.0 { 0.0 } else { theme::SPIN };
        }
    });

    coins.for_each_mut(|entity, (perch, _bob, _spin)| {
        let out = if entity == chosen {
            theme::STEP_OUT
        } else {
            0.0
        };
        if perch.0.z != out {
            perch.0.z = out;
        }
    });
}

pub fn watch_the_hover(
    time: Res<Time>,
    row: Res<Row>,
    settings: Res<HoverSettings>,
    mut readout: ResMut<Readout>,
    mut coins: Query<(&Perch, &Bob, &Phase, &Spin, &LocalTransform)>,
) {
    let clock = time.uptime_seconds as f32;
    readout.notes.clear();
    readout.notes.push(format!("{} hovering", row.coins.len()));
    readout.notes.push(format!("rate {:.1}", settings.rate));
    let Some(chosen) = row.coins.get(row.chosen).copied() else {
        return;
    };
    readout.notes.push(format!("coin {}", row.chosen + 1));
    coins.for_entity(chosen, |(perch, bob, phase, spin, transform)| {
        readout.notes.push(format!("bob {:.2}", bob.0));
        readout.notes.push(format!(
            "lift {:.2} off a perch of {:.2}",
            lift(bob, phase, &settings, clock),
            perch.0.y
        ));
        readout
            .notes
            .push(format!("standing at {:.2}", transform.translation.y));
        readout
            .notes
            .push(format!("spin {}", if spin.0 != 0.0 { "on" } else { "off" }));
    });
}
