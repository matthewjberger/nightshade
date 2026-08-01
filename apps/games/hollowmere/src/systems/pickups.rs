use crate::commands::audio::cue_one_of;
use crate::commands::chat::system_line;
use crate::commands::effects::burst;
use crate::commands::{inventory_add, toast_show};
use crate::components::{Cooldowns, Home, Pickup, Regrow};
use crate::data::{GATHER_CLIPS, Item, PICKUPS};
use crate::queries::flat_distance;
use crate::resources::{BurstKind, Bursts, Cues, Hero, Inventory, Journal, Toast};
use crate::theme;
use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::LocalTransform;

pub fn gather(
    time: Res<Time>,
    mut hero: ResMut<Hero>,
    mut journal: ResMut<Journal>,
    mut inventory: ResMut<Inventory>,
    mut toast: ResMut<Toast>,
    mut cues: ResMut<Cues>,
    mut bursts: ResMut<Bursts>,
    players: Query<(&LocalTransform,), With<Cooldowns>>,
    mut nodes: Query<(&Pickup, &mut Regrow, &Home, &mut LocalTransform)>,
) {
    let delta_time = time.delta_time;
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
        if flat_distance(position, home.0) > theme::PICKUP_REACH {
            return;
        }
        regrow.0 = PICKUPS[pickup.0].regrow;
        transform.translation.y -= theme::SINK_DEPTH;
        picked.push((pickup.0, home.0));
    });

    for (kind, spot) in picked {
        burst(
            &mut bursts,
            BurstKind::Gather,
            spot + Vec3::new(0.0, 0.6, 0.0),
        );
        cue_one_of(
            &mut cues,
            &mut hero.random,
            &GATHER_CLIPS,
            theme::SOUND_PLAIN,
        );
        let leftover = inventory_add(&mut inventory, Item::Loot(kind), 1);
        let name = PICKUPS[kind].name;
        if leftover > 0 {
            toast_show(&mut toast, format!("No room in the pocket for {name}"));
        } else {
            system_line(&mut journal.chat, format!("You gather {name}."));
        }
    }
}
