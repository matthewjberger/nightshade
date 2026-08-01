use crate::commands::audio::{cue, cue_one_of};
use crate::commands::chat::system_line;
use crate::commands::effects::burst;
use crate::components::{Cooldowns, Home, Pickup, Regrow};
use crate::data::{
    GATHER_CLIPS, GIVERS, Objective, PICKUP_REACH, PICKUPS, QUESTS, SINK_DEPTH, SOUND_DONE,
    SOUND_PLAIN, SOUND_QUIET,
};
use crate::queries::flat_distance;
use crate::resources::{BurstKind, Bursts, Cues, ThornmereResources};
use crate::systems::quests::turn_in_giver;
use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::LocalTransform;

pub fn gather(
    time: Res<Time>,
    mut resources: ResMut<ThornmereResources>,
    mut cues: ResMut<Cues>,
    mut bursts: ResMut<Bursts>,
    players: Query<(&LocalTransform,), With<Cooldowns>>,
    mut nodes: Query<(&Pickup, &mut Regrow, &Home, &mut LocalTransform)>,
) {
    let delta_time = time.delta_time;
    let resources = &mut *resources;
    let mut position = None;
    players.for_each(|_entity, (transform,)| position = Some(transform.translation));

    let mut picked = Vec::new();
    nodes.for_each_mut(|_entity, (pickup, regrow, home, transform)| {
        if regrow.0 > 0.0 {
            regrow.0 -= delta_time;
            if regrow.0 <= 0.0 {
                regrow.0 = 0.0;
                transform.translation = home.0;
            }
            return;
        }
        let Some(position) = position else {
            return;
        };
        if flat_distance(position, home.0) > PICKUP_REACH {
            return;
        }
        let wanted = resources.quests.active.iter().any(|state| {
            matches!(
                QUESTS[state.quest].objective,
                Objective::Gather { pickup: kind, .. } if kind == pickup.0
            ) && !state.complete
        });
        if !wanted {
            return;
        }
        regrow.0 = PICKUPS[pickup.0].regrow;
        transform.translation.y -= SINK_DEPTH;
        picked.push((pickup.0, home.0));
    });

    for (kind, spot) in picked {
        burst(
            &mut bursts,
            BurstKind::Gather,
            spot + Vec3::new(0.0, 0.6, 0.0),
        );
        cue_one_of(&mut cues, &mut resources.random, &GATHER_CLIPS, SOUND_PLAIN);
        let mut notes = Vec::new();
        for state in resources.quests.active.iter_mut() {
            let Objective::Gather { pickup, count } = QUESTS[state.quest].objective else {
                continue;
            };
            if pickup != kind || state.complete {
                continue;
            }
            state.progress += 1;
            if state.progress >= count {
                state.complete = true;
                cue(&mut cues, SOUND_DONE, SOUND_QUIET);
                notes.push(format!(
                    "{} complete. Return to {}.",
                    QUESTS[state.quest].title,
                    GIVERS[turn_in_giver(state.quest)].name
                ));
            } else {
                notes.push(format!(
                    "You gather {}: {}/{}",
                    PICKUPS[kind].name, state.progress, count
                ));
            }
        }
        for note in notes {
            system_line(&mut resources.chat, note);
        }
    }
}
