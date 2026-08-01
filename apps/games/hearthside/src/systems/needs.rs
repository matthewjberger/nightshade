use crate::components::{Needs, Using};
use crate::contract::{HearthsideCommand, HearthsideCommands, HearthsideEvent};
use crate::data;
use crate::queries;
use crate::resources::{Grid, Lot};
use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::LocalTransform;

pub fn decay(time: Res<Time>, dwellers: Query<(&Using, &mut Needs)>) {
    let delta_time = time.delta_time;
    dwellers.for_each(|_entity, (using, needs)| {
        let resting = using.0.is_some();
        for (index, level) in needs.0.iter_mut().enumerate() {
            let rate = if resting {
                data::NEED_DECAY[index] * 0.35
            } else {
                data::NEED_DECAY[index]
            };
            *level = (*level - rate * delta_time).max(0.0);
        }
    });
}

pub fn earn(
    time: Res<Time>,
    grid: Res<Grid>,
    mut lot: ResMut<Lot>,
    dwellers: Query<(&Needs, &LocalTransform)>,
) {
    let delta_time = time.delta_time;
    let mut rate = 0.0;
    dwellers.for_each(|_entity, (needs, transform)| {
        let indoors =
            queries::point_to_cell(transform.translation).is_some_and(|cell| grid.floored[cell]);
        rate += queries::wage_rate(needs, indoors);
    });
    lot.wage = rate;
    lot.money += rate * delta_time;
}

pub fn apply_commands(
    mut commands: ResMut<HearthsideCommands>,
    mut lot: ResMut<Lot>,
    mut events: EventWriter<HearthsideEvent>,
    dwellers: Query<(&mut Needs,)>,
) {
    let mut filling = false;
    for command in std::mem::take(&mut commands.pending) {
        match command {
            HearthsideCommand::Grant { amount } => {
                lot.money += amount as f32;
                events.send(HearthsideEvent::Granted { amount });
            }
            HearthsideCommand::FillNeeds => {
                filling = true;
                events.send(HearthsideEvent::NeedsFilled);
            }
        }
    }

    if filling {
        dwellers.for_each(|_entity, (needs,)| {
            needs.0 = [data::NEED_FULL; 4];
        });
    }
}
