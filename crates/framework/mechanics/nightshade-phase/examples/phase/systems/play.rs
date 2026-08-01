use crate::resources::Town;
use crate::theme;
use nightshade::ecs::system_param::EventReader;
use nightshade::prelude::*;
use nightshade_phase::commands::{turn_over, turn_to};
use nightshade_phase::queries::{at, faces, look};
use nightshade_phase::{Phase, PhaseSettings, PhaseTurned};
use nightshade_platform::contract::input::Input;
use nightshade_platform::prelude::KeyCode;
use nightshade_readout::Readout;

pub fn take_the_orders(input: Res<Input>, settings: Res<PhaseSettings>, mut phase: ResMut<Phase>) {
    if input.keyboard.just_pressed(KeyCode::Space) {
        turn_over(&mut phase, &settings);
    }
    for (digit, key) in [KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3]
        .into_iter()
        .enumerate()
    {
        if input.keyboard.just_pressed(key) && digit < faces(&settings) {
            turn_to(&mut phase, digit);
        }
    }
}

pub fn hear_the_turn(turned: EventReader<PhaseTurned>, mut town: ResMut<Town>) {
    for event in turned.iter() {
        town.turns += 1;
        town.said = format!(
            "the town turns to {}",
            theme::FACES
                .get(event.at)
                .copied()
                .unwrap_or("something else")
        );
    }
}

pub fn show_the_town(
    phase: Res<Phase>,
    settings: Res<PhaseSettings>,
    town: Res<Town>,
    mut readout: ResMut<Readout>,
) {
    readout.notes.clear();
    readout
        .notes
        .push(format!("face {} of {}", at(&phase) + 1, faces(&settings)));
    if let Some(wanted) = look(&settings, at(&phase)) {
        readout
            .notes
            .push(format!("fog ends at {:.0}", wanted.fog_end));
        readout.notes.push(format!(
            "lamps at {:.2}, {:.2}, {:.2}",
            wanted.light_color.x, wanted.light_color.y, wanted.light_color.z
        ));
    }
    readout.notes.push(format!("{} turns", town.turns));
    if !town.said.is_empty() {
        readout.notes.push(town.said.clone());
    }
}
