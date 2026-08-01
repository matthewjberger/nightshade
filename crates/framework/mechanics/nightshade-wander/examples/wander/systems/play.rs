use crate::theme;
use nightshade::ecs::system_param::{Query, Res, ResMut};
use nightshade_plane::PlayPlane;
use nightshade_platform::contract::input::Input;
use nightshade_platform::prelude::KeyCode;
use nightshade_random::Random;
use nightshade_readout::Readout;
use nightshade_wander::commands::{pick_a_spot, rest_a_while};
use nightshade_wander::queries::resting;
use nightshade_wander::{Home, Resting, Roaming, WanderSettings, Wandering};

pub fn take_the_orders(
    input: Res<Input>,
    settings: Res<WanderSettings>,
    plane: Res<PlayPlane>,
    mut roll: ResMut<Random>,
    walkers: Query<(&Home, &mut Wandering, &mut Resting, &mut Roaming)>,
) {
    let nudged = input.keyboard.just_pressed(KeyCode::Space);
    let sent_to_rest = input.keyboard.just_pressed(KeyCode::KeyR);
    let slower = input.keyboard.just_pressed(KeyCode::KeyQ);
    let faster = input.keyboard.just_pressed(KeyCode::KeyE);
    if !nudged && !sent_to_rest && !slower && !faster {
        return;
    }
    walkers.for_each(|_entity, (home, wandering, rest, roaming)| {
        if slower {
            roaming.0 = (roaming.0 - theme::PACE_STEP).max(0.0);
        }
        if faster {
            roaming.0 = (roaming.0 + theme::PACE_STEP).min(theme::PACE_MOST);
        }
        if nudged {
            rest.0 = 0.0;
            pick_a_spot(plane.0, wandering, home, &settings, &mut roll);
        }
        if sent_to_rest {
            rest_a_while(rest, &settings, &mut roll);
        }
    });
}

pub fn count_the_resting(mut readout: ResMut<Readout>, walkers: Query<(&Resting, &Roaming)>) {
    let mut walking = 0;
    let mut waiting = 0;
    let mut pace = 0.0;
    walkers.for_each(|_entity, (rest, roaming)| {
        pace = roaming.0;
        if resting(rest) {
            waiting += 1;
        } else {
            walking += 1;
        }
    });
    readout.notes.clear();
    readout.notes.push(format!("{walking} walking"));
    readout.notes.push(format!("{waiting} resting"));
    readout.notes.push(format!("pace {pace:.2}"));
}
